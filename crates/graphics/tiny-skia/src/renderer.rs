use crate::error::TinySkiaError;
use crate::handle::{
    SurfaceSlotId, TinySkiaBrushHandle, TinySkiaImageHandle, TinySkiaSurfaceId,
    TinySkiaTextFormatHandle,
};
use flor_base::graphics::{
    Gradient, ImageDrawOptions, LayoutText, ParagraphAlignment, Path, PathDrawOptions, Render,
    RenderContext, SurfaceDrawOptions, TextDrawOptions, FONT_SYSTEM,
};
use flor_base::types::{Color, Rect, Transform2D};
use libblur::BlurError;
use slotmap::SlotMap;
use tiny_skia::{Paint, Pixmap, PixmapMut, PixmapPaint, PremultipliedColorU8};
#[cfg(feature = "svg")]
use {
    crate::handle::{SvgSlotId, TinySkiaSvgHandle},
    flor_base::graphics::SvgDrawOptions,
};

mod config;
use crate::display_context::{DisplayContext, NativeDisplayContext};
use crate::has_transform::HasTransform;
use crate::text_layout::TinySkiaTextLayout;
use crate::to_tiny_skia::ToTinySkia;
pub use config::*;
use flor_base::graphics;

#[derive(Debug)]
pub struct TinySkiaRenderer {
    pub context: NativeDisplayContext,
    pub default_pixmap: Pixmap,
    pub surface_pixmap: SlotMap<SurfaceSlotId, Pixmap>,
    #[cfg(feature = "svg")]
    pub svg_pixmap: SlotMap<SvgSlotId, Pixmap>,
    pub wait_v_sync: bool,
    pub transform_stack: Vec<tiny_skia::Transform>,
    pub current_render_target: Option<SurfaceSlotId>,

    // Text rendering caches
    pub swash_cache: cosmic_text::SwashCache,

    // Clipping state
    pub clip_stack: Vec<(tiny_skia::Path, Option<tiny_skia::Mask>)>,
    pub active_clip: Option<tiny_skia::Mask>,
    pub clip_suspended: bool,

    // Scale factor
    pub dpi_scale: (f32, f32),
}

impl Render for TinySkiaRenderer {
    #[cfg(target_os = "windows")]
    type HWND = windows::Win32::Foundation::HWND;
    type Render = TinySkiaRenderer;
    type Config = TinySkiaConfig;

    fn create(
        hwnd: impl Into<Self::HWND>,
        width: u32,
        height: u32,
        wait_v_sync: bool,
        config: Self::Config,
    ) -> Result<Self::Render, Self::Error> {
        let hwnd = hwnd.into();
        let context = NativeDisplayContext::create(hwnd, config)?;

        Ok(TinySkiaRenderer {
            context,
            default_pixmap: Pixmap::new(width.max(1), height.max(1))
                .unwrap_or_else(|| unreachable!()),
            surface_pixmap: Default::default(),
            #[cfg(feature = "svg")]
            svg_pixmap: Default::default(),
            wait_v_sync,
            transform_stack: vec![tiny_skia::Transform::identity()],
            current_render_target: None,
            swash_cache: cosmic_text::SwashCache::new(),
            clip_stack: Vec::new(),
            active_clip: None,
            clip_suspended: false,
            dpi_scale: (1.0, 1.0),
        })
    }
}

impl TinySkiaRenderer {
    fn fast_blur(
        pixels: &mut [PremultipliedColorU8],
        width: usize,
        height: usize,
        radius: u32,
    ) -> Result<(), TinySkiaError> {
        if radius == 0 || width == 0 || height == 0 {
            return Err(TinySkiaError::BlurError(BlurError::InvalidArguments));
        }

        let byte_slice = unsafe {
            std::slice::from_raw_parts_mut(pixels.as_mut_ptr() as *mut u8, pixels.len() * 4)
        };

        let mut dst_image = libblur::BlurImageMut::borrow(
            byte_slice,
            width as u32,
            height as u32,
            libblur::FastBlurChannels::Channels4,
        );

        libblur::fast_gaussian_next(
            &mut dst_image,
            libblur::AnisotropicRadius::new(radius),
            libblur::ThreadingPolicy::Adaptive,
            libblur::EdgeMode2D::new(libblur::EdgeMode::Clamp),
        )?;
        Ok(())
    }

    fn get_current_transform(&self) -> tiny_skia::Transform {
        self.transform_stack
            .last()
            .copied()
            .unwrap_or(tiny_skia::Transform::identity())
    }

    fn get_options_transform<T: HasTransform>(&self, options: Option<&T>) -> tiny_skia::Transform {
        let current = self.get_current_transform();
        options
            .and_then(|opts| opts.get_transform())
            .map(|t| current.pre_concat(t.to_tiny_skia()))
            .unwrap_or(current)
    }

    /// Run a closure with the current render target PixmapMut and the active ClipMask
    fn with_current_pixmap<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut PixmapMut, Option<&tiny_skia::Mask>) -> R,
    {
        let clip = self.active_clip.as_ref();
        if let Some(target) = self.current_render_target {
            if let Some(surface) = self.surface_pixmap.get_mut(target) {
                return f(&mut surface.as_mut(), clip);
            }
        }
        f(&mut self.default_pixmap.as_mut(), clip)
    }

    /// Helper to get swash cache and current pixmap simultaneously without closure capture issues
    fn split_pixmap_and_cache(
        &'_ mut self,
    ) -> (
        &'_ mut cosmic_text::SwashCache,
        PixmapMut<'_>,
        Option<&tiny_skia::Mask>,
    ) {
        let clip = self.active_clip.as_ref();

        let pixmap = if let Some(target) = self.current_render_target {
            if let Some(surface) = self.surface_pixmap.get_mut(target) {
                surface.as_mut()
            } else {
                self.default_pixmap.as_mut()
            }
        } else {
            self.default_pixmap.as_mut()
        };

        (&mut self.swash_cache, pixmap, clip)
    }

    fn build_tiny_skia_path(path: &Path) -> Option<tiny_skia::Path> {
        use flor_base::graphics::PathCommand;
        let mut pb = tiny_skia::PathBuilder::new();
        for cmd in path.commands() {
            match cmd {
                PathCommand::MoveTo(x, y) => pb.move_to(*x, *y),
                PathCommand::LineTo(x, y) => pb.line_to(*x, *y),
                PathCommand::Bezier(pts) => {
                    match pts.len() {
                        2 => {
                            pb.quad_to(pts[0].0, pts[0].1, pts[1].0, pts[1].1);
                        }
                        3 => {
                            pb.cubic_to(pts[0].0, pts[0].1, pts[1].0, pts[1].1, pts[2].0, pts[2].1);
                        }
                        _ => {
                            // Unsupported bezier degree, fallback to line to endpoint
                            if let Some(last) = pts.last() {
                                pb.line_to(last.0, last.1);
                            }
                        }
                    }
                }
                PathCommand::Close => pb.close(),
            }
        }
        pb.finish()
    }

    fn draw_shadow(
        &mut self,
        path: &Path,
        shadow: &graphics::Shadow,
        options_transform: Option<&Transform2D>,
    ) -> Result<(), TinySkiaError> {
        let ts_path = if let Some(p) = Self::build_tiny_skia_path(path) {
            p
        } else {
            return Ok(());
        };

        if shadow.blur_radius <= 0.0 {
            return Ok(());
        }

        let transform = if let Some(t) = options_transform {
            self.get_current_transform().pre_concat(t.to_tiny_skia())
        } else {
            self.get_current_transform()
        };
        let transform = transform.post_translate(shadow.offset_x, shadow.offset_y);

        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.set_color(shadow.color.to_tiny_skia());

        let bounds = ts_path.bounds();
        let pad = shadow.blur_radius.ceil() * 2.0;

        let left = (bounds.left() - pad).floor();
        let top = (bounds.top() - pad).floor();
        let right = (bounds.right() + pad).ceil();
        let bottom = (bounds.bottom() + pad).ceil();

        let width = (right - left) as u32;
        let height = (bottom - top) as u32;

        if width == 0 || height == 0 {
            return Ok(());
        }

        if let Some(mut blur_pixmap) = Pixmap::new(width, height) {
            let mut local_transform = transform;
            local_transform = local_transform.post_translate(-left, -top);

            blur_pixmap.fill_path(
                &ts_path,
                &paint,
                tiny_skia::FillRule::Winding,
                local_transform,
                None,
            );

            Self::fast_blur(
                blur_pixmap.pixels_mut(),
                width as usize,
                height as usize,
                shadow.blur_radius.round() as u32,
            )?;

            let mut draw_transform = tiny_skia::Transform::identity();
            draw_transform = draw_transform.post_translate(left, top);

            self.with_current_pixmap(|p, clip| {
                p.draw_pixmap(
                    0,
                    0,
                    blur_pixmap.as_ref(),
                    &PixmapPaint::default(),
                    draw_transform,
                    clip,
                );
            });
        }

        Ok(())
    }

    fn update_clip_mask(&mut self) {
        if self.clip_suspended || self.clip_stack.is_empty() {
            self.active_clip = None;
            return;
        }

        if let Some((_, Some(ref mask))) = self.clip_stack.last() {
            self.active_clip = Some(mask.clone());
        } else {
            self.active_clip = None;
        }
    }

    fn push_path_internal(&mut self, ts_path: tiny_skia::Path) {
        let width = self.default_pixmap.width();
        let height = self.default_pixmap.height();

        let prev_mask = self.clip_stack.last().and_then(|(_, m)| m.as_ref());

        if let Some(mut new_mask) = tiny_skia::Mask::new(width, height) {
            if let Some(prev) = prev_mask {
                new_mask.clone_from(prev);
                new_mask.intersect_path(
                    &ts_path,
                    tiny_skia::FillRule::Winding,
                    true,
                    tiny_skia::Transform::identity(),
                );
            } else {
                new_mask.fill_path(
                    &ts_path,
                    tiny_skia::FillRule::Winding,
                    true,
                    tiny_skia::Transform::identity(),
                );
            }
            self.clip_stack.push((ts_path, Some(new_mask)));
        } else {
            self.clip_stack.push((ts_path, None));
        }

        self.update_clip_mask();
    }

    fn set_brush(brush: &TinySkiaBrushHandle, paint: &mut Paint) {
        match brush {
            TinySkiaBrushHandle::Solid(color) => {
                paint.set_color(*color);
            }
            TinySkiaBrushHandle::Gradient { gradient } => {
                let mut stops = Vec::new();
                match gradient {
                    Gradient::Linear { start, end, colors } => {
                        for (pos, c) in colors {
                            stops.push(tiny_skia::GradientStop::new(*pos, c.to_tiny_skia()));
                        }
                        if let Some(shader) = tiny_skia::LinearGradient::new(
                            tiny_skia::Point::from_xy(start.0, start.1),
                            tiny_skia::Point::from_xy(end.0, end.1),
                            stops,
                            tiny_skia::SpreadMode::Pad,
                            tiny_skia::Transform::identity(),
                        ) {
                            paint.shader = shader;
                        } else {
                            paint.set_color_rgba8(0, 0, 0, 255);
                        }
                    }
                    Gradient::Radial {
                        center,
                        radius,
                        colors,
                    } => {
                        for (pos, c) in colors {
                            stops.push(tiny_skia::GradientStop::new(*pos, c.to_tiny_skia()));
                        }
                        if let Some(shader) = tiny_skia::RadialGradient::new(
                            tiny_skia::Point::from_xy(center.0, center.1),
                            0.0,
                            tiny_skia::Point::from_xy(center.0, center.1),
                            *radius,
                            stops,
                            tiny_skia::SpreadMode::Pad,
                            tiny_skia::Transform::identity(),
                        ) {
                            paint.shader = shader;
                        } else {
                            paint.set_color_rgba8(0, 0, 0, 255);
                        }
                    }
                }
            }
        }
    }

    fn convert_rgba_to_premultiplied(src_rgba: &[u8], dst_premultiplied: &mut [u8]) {
        for (src, dst) in src_rgba
            .chunks_exact(4)
            .zip(dst_premultiplied.chunks_exact_mut(4))
        {
            let r = src[0];
            let g = src[1];
            let b = src[2];
            let a = src[3];
            let color = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
            dst[0] = color.red();
            dst[1] = color.green();
            dst[2] = color.blue();
            dst[3] = color.alpha();
        }
    }

    fn recolor_shadow(pixmap: &mut Pixmap, color_r: u32, color_g: u32, color_b: u32, color_a: f32) {
        for pixel in pixmap.pixels_mut() {
            let a = pixel.alpha() as f32 / 255.0;
            let final_a = a * color_a;
            if final_a > 0.0 {
                let fa8 = (final_a * 255.0).round() as u8;
                let fr8 = (color_r as f32 * final_a).round() as u8;
                let fg8 = (color_g as f32 * final_a).round() as u8;
                let fb8 = (color_b as f32 * final_a).round() as u8;

                *pixel = PremultipliedColorU8::from_rgba(fr8, fg8, fb8, fa8)
                    .unwrap_or(PremultipliedColorU8::TRANSPARENT);
            } else {
                *pixel = PremultipliedColorU8::TRANSPARENT;
            }
        }
    }

    fn fill_rect_with_transform(
        &mut self,
        offset_x: f32,
        offset_y: f32,
        sw: f32,
        sh: f32,
        paint: &Paint,
        transform: tiny_skia::Transform,
    ) {
        if let Some(rect) = tiny_skia::Rect::from_xywh(offset_x, offset_y, sw, sh) {
            self.with_current_pixmap(|p, clip| {
                p.fill_rect(rect, paint, transform, clip);
            });
        }
    }

    fn apply_opacity(paint: &mut Paint, opacity: Option<f32>) {
        if let Some(op) = opacity {
            paint.set_color_rgba8(255, 255, 255, (op * 255.0).clamp(0.0, 255.0) as u8);
        }
    }

    #[cfg(feature = "svg")]
    fn clone_pixmap(pixmap: &Pixmap) -> Option<Pixmap> {
        let mut clone = Pixmap::new(pixmap.width(), pixmap.height())?;
        clone.data_mut().copy_from_slice(pixmap.data());
        Some(clone)
    }

    fn get_glyph_pixel(
        glyph_color: Option<tiny_skia::Color>,
        a: f32,
        fa: u8,
    ) -> PremultipliedColorU8 {
        if let Some(c) = glyph_color {
            let mut p_color = c;
            p_color.apply_opacity(a);
            p_color.premultiply().to_color_u8()
        } else {
            PremultipliedColorU8::from_rgba(fa, fa, fa, fa)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT)
        }
    }
}

struct FuncTimer(&'static str, std::time::Instant);
impl FuncTimer {
    fn new(name: &'static str) -> Self {
        Self(name, std::time::Instant::now())
    }
}
impl Drop for FuncTimer {
    fn drop(&mut self) {
        // let elapsed = self.1.elapsed();
        // println!(
        //     "[RenderContext] TinySkiaRenderer::{} took {:?}",
        //     self.0, elapsed
        // );
    }
}
impl RenderContext for TinySkiaRenderer {
    type Error = TinySkiaError;
    type ImageHandle = TinySkiaImageHandle;
    type SurfaceId = TinySkiaSurfaceId;
    type BrushHandle = TinySkiaBrushHandle;
    #[cfg(feature = "svg")]
    type SvgHandle = TinySkiaSvgHandle;
    type TextFormatHandle = TinySkiaTextFormatHandle;
    type LayoutText = TinySkiaTextLayout;

    fn begin(&mut self) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("begin");
        Ok(())
    }

    fn end(&mut self) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("end");
        self.context.present(
            self.default_pixmap.width(),
            self.default_pixmap.height(),
            self.default_pixmap.data(),
        )?;
        if self.wait_v_sync {
            self.context.wait_v_sync();
        }
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("clear");
        self.with_current_pixmap(|p, clip| {
            if let Some(c) = clip {
                let mut paint = Paint::default();
                paint.set_color(color.to_tiny_skia());
                p.fill_rect(
                    tiny_skia::Rect::from_xywh(0.0, 0.0, p.width() as f32, p.height() as f32)
                        .unwrap(),
                    &paint,
                    tiny_skia::Transform::identity(),
                    Some(c),
                );
            } else {
                p.fill(color.to_tiny_skia());
            }
        });
        Ok(())
    }

    fn test(&mut self) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("test");
        Ok(())
    }

    fn update_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("update_window_size");
        if let Some(pixmap) = Pixmap::new(width.max(1), height.max(1)) {
            self.default_pixmap = pixmap;
            self.clip_stack.clear();
            self.active_clip = None;
        }
        Ok(())
    }
    fn set_scale_factor(&mut self, dpi_x: f32, dpi_y: f32) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("set_scale_factor");
        self.dpi_scale = (dpi_x, dpi_y);
        Ok(())
    }

    fn create_surface(&mut self, width: u32, height: u32) -> Result<Self::SurfaceId, Self::Error> {
        let _timer = FuncTimer::new("create_surface");
        let pixmap =
            Pixmap::new(width.max(1), height.max(1)).ok_or(TinySkiaError::CreateSurfaceError)?;
        let id = self.surface_pixmap.insert(pixmap);
        Ok(TinySkiaSurfaceId { id })
    }

    fn set_render_target(&mut self, surface_id: &Self::SurfaceId) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("set_render_target");
        if self.surface_pixmap.contains_key(surface_id.id) {
            self.current_render_target = Some(surface_id.id);
            Ok(())
        } else {
            Err(TinySkiaError::SurfaceNotFoundError)
        }
    }

    fn reset_render_target(&mut self) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("reset_render_target");
        self.current_render_target = None;
        Ok(())
    }

    fn create_image_from_bytes(&mut self, bytes: &[u8]) -> Result<Self::ImageHandle, Self::Error> {
        let _timer = FuncTimer::new("create_image_from_bytes");
        let img = image::load_from_memory(bytes).map_err(|_| TinySkiaError::ImageDecodeError)?;
        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        let mut pixmap = Pixmap::new(width, height).ok_or(TinySkiaError::CreateSurfaceError)?;

        let slice = pixmap.data_mut();
        Self::convert_rgba_to_premultiplied(&rgba, slice);

        Ok(TinySkiaImageHandle {
            width,
            height,
            delays: vec![0],
            total_delays: 0,
            frames: std::sync::Arc::new(vec![pixmap]),
        })
    }

    fn create_image_from_raw_bytes(
        &mut self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<Self::ImageHandle, Self::Error> {
        let _timer = FuncTimer::new("create_image_from_raw_bytes");
        let mut frames = Vec::with_capacity(raw_bytes.len());
        for frame_bytes in raw_bytes {
            let mut pixmap = Pixmap::new(width, height).ok_or(TinySkiaError::CreateSurfaceError)?;
            let slice = pixmap.data_mut();

            Self::convert_rgba_to_premultiplied(frame_bytes, slice);
            frames.push(pixmap);
        }

        let total_delays = delays.iter().map(|&d| d as u128).sum();

        Ok(TinySkiaImageHandle {
            width,
            height,
            delays,
            total_delays,
            frames: std::sync::Arc::new(frames),
        })
    }

    #[cfg(feature = "svg")]
    fn create_svg(&mut self, bytes: &[u8]) -> Result<Self::SvgHandle, Self::Error> {
        let _timer = FuncTimer::new("create_svg");
        let opt = usvg::Options::default();
        let tree =
            usvg::Tree::from_data(bytes, &opt).map_err(|_| TinySkiaError::CreateSurfaceError)?;

        let size = tree.size();
        Ok(TinySkiaSvgHandle {
            width: size.width().ceil() as u32,
            height: size.height().ceil() as u32,
            tree: std::sync::Arc::new(tree),
            cache: std::sync::Arc::new(parking_lot::RwLock::new(crate::handle::SvgCache {
                svg_pixmap: None,
                shadow_pixmap: None,
            })),
        })
    }

    fn create_text_format(
        &mut self,
        font_family_name: &str,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        let _timer = FuncTimer::new("create_text_format");
        Ok(TinySkiaTextFormatHandle::new_with_font_family_name(
            font_family_name,
        ))
    }

    #[cfg(feature = "memory-font")]
    fn create_text_format_from_bytes(
        &mut self,
        font_data: &[u8],
        _ttc_index: u32,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        let _timer = FuncTimer::new("create_text_format_from_bytes");
        let source = cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(font_data.to_vec()));
        Ok(TinySkiaTextFormatHandle::new_with_font_source(source))
    }

    fn create_text_layout(
        &self,
        text: String,
        bounds: Rect<f32>,
        text_format: Self::TextFormatHandle,
    ) -> Result<Self::LayoutText, Self::Error> {
        let _timer = FuncTimer::new("create_text_layout");
        let mut layout =
            crate::text_layout::TinySkiaTextLayout::create_text_layout(text, bounds, text_format);
        layout.set_dpi_scale(self.dpi_scale.0, self.dpi_scale.1);
        Ok(layout)
    }

    fn create_solid_color_brush(
        &mut self,
        color: Color,
        opacity: Option<f32>,
    ) -> Result<Self::BrushHandle, Self::Error> {
        let _timer = FuncTimer::new("create_solid_color_brush");
        let mut tiny_color = color.to_tiny_skia();
        if let Some(op) = opacity {
            tiny_color.apply_opacity(op);
        }
        Ok(TinySkiaBrushHandle::Solid(tiny_color))
    }

    fn create_gradient_brush(
        &mut self,
        gradient: &Gradient,
    ) -> Result<Self::BrushHandle, Self::Error> {
        let _timer = FuncTimer::new("create_gradient_brush");
        Ok(TinySkiaBrushHandle::Gradient {
            gradient: gradient.clone(),
        })
    }

    fn draw_image(
        &mut self,
        handle: &Self::ImageHandle,
        x: f32,
        y: f32,
        width: Option<f32>,
        height: Option<f32>,
        options: Option<&ImageDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("draw_image");
        let target_width = width.unwrap_or(handle.width as f32);
        let target_height = height.unwrap_or(handle.height as f32);

        let mut paint = Paint::default();
        paint.anti_alias = true;

        let mut shader_transform = tiny_skia::Transform::identity();
        let scale_x = target_width / handle.width as f32;
        let scale_y = target_height / handle.height as f32;
        shader_transform = shader_transform
            .pre_translate(x, y)
            .pre_scale(scale_x, scale_y);

        Self::apply_opacity(&mut paint, options.and_then(|o| o.opacity));

        let mut pb = tiny_skia::PathBuilder::new();
        if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, target_width, target_height) {
            pb.push_rect(rect);
        } else {
            return Ok(());
        }
        let path = if let Some(p) = pb.finish() {
            p
        } else {
            return Ok(());
        };

        let transform = self.get_options_transform(options);

        let frame_index = options.and_then(|o| o.frame_index).unwrap_or(0);
        let frames = &handle.frames;
        let pixmap = frames.get(frame_index).unwrap_or_else(|| &frames[0]);

        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let pad = (shadow.blur_radius.ceil() * 2.5).max(0.0);
                let sw = (target_width + pad * 2.0).ceil() as u32;
                let sh = (target_height + pad * 2.0).ceil() as u32;

                if sw > 0 && sh > 0 {
                    if let Some(mut shadow_pixmap) = Pixmap::new(sw, sh) {
                        shadow_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                        let shadow_render_transform =
                            tiny_skia::Transform::from_translate(pad, pad)
                                .pre_scale(scale_x, scale_y);

                        let img_pattern = tiny_skia::Pattern::new(
                            pixmap.as_ref(),
                            tiny_skia::SpreadMode::Pad,
                            tiny_skia::FilterQuality::Bilinear,
                            1.0,
                            shadow_render_transform,
                        );

                        let mut img_paint = Paint::default();
                        img_paint.shader = img_pattern;
                        img_paint.anti_alias = true;

                        if let Some(rect) =
                            tiny_skia::Rect::from_xywh(pad, pad, target_width, target_height)
                        {
                            let p = tiny_skia::PathBuilder::from_rect(rect);
                            shadow_pixmap.fill_path(
                                &p,
                                &img_paint,
                                tiny_skia::FillRule::Winding,
                                tiny_skia::Transform::identity(),
                                None,
                            );
                        }

                        let color_r = shadow.color.r as u32;
                        let color_g = shadow.color.g as u32;
                        let color_b = shadow.color.b as u32;
                        let color_a = shadow.color.a as f32 / 255.0;

                        Self::recolor_shadow(
                            &mut shadow_pixmap,
                            color_r,
                            color_g,
                            color_b,
                            color_a,
                        );

                        if shadow.blur_radius > 0.0 {
                            Self::fast_blur(
                                shadow_pixmap.pixels_mut(),
                                sw as usize,
                                sh as usize,
                                shadow.blur_radius.round() as u32,
                            )?;
                        }

                        let offset_x = x - pad + shadow.offset_x;
                        let offset_y = y - pad + shadow.offset_y;

                        let pattern = tiny_skia::Pattern::new(
                            shadow_pixmap.as_ref(),
                            tiny_skia::SpreadMode::Pad,
                            tiny_skia::FilterQuality::Bilinear,
                            1.0,
                            tiny_skia::Transform::from_translate(offset_x, offset_y),
                        );

                        let mut paint = Paint::default();
                        paint.shader = pattern;
                        paint.anti_alias = true;

                        self.fill_rect_with_transform(
                            offset_x, offset_y, sw as f32, sh as f32, &paint, transform,
                        );
                    }
                }
            }
        }

        let pattern = tiny_skia::Pattern::new(
            pixmap.as_ref(),
            tiny_skia::SpreadMode::Pad,
            tiny_skia::FilterQuality::Bilinear,
            1.0,
            shader_transform,
        );
        paint.shader = pattern;

        if let Some(target) = self.current_render_target {
            if let Some(surface) = self.surface_pixmap.get_mut(target) {
                surface.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    transform,
                    self.active_clip.as_ref(),
                );
                return Ok(());
            }
        }
        self.default_pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            transform,
            self.active_clip.as_ref(),
        );

        Ok(())
    }

    #[cfg(feature = "svg")]
    fn draw_svg(
        &mut self,
        handle: &Self::SvgHandle,
        x: f32,
        y: f32,
        width: Option<f32>,
        height: Option<f32>,
        options: Option<&SvgDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("draw_svg");
        let target_width = width.unwrap_or(handle.width as f32);
        let target_height = height.unwrap_or(handle.height as f32);

        if target_width <= 0.0 || target_height <= 0.0 {
            return Ok(());
        }

        let scale_x = target_width / handle.width as f32;
        let scale_y = target_height / handle.height as f32;

        let mut transform = self.get_current_transform();
        if let Some(opts) = options {
            if let Some(t) = &opts.transform {
                transform = transform.pre_concat(t.to_tiny_skia());
            }
        }

        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let pad = (shadow.blur_radius.ceil() * 2.5).max(0.0);

                let sw = (target_width + pad * 2.0).ceil() as u32;
                let sh = (target_height + pad * 2.0).ceil() as u32;

                if sw > 0 && sh > 0 {
                    let mut cached_shadow = None;

                    {
                        let cache = handle.cache.read();
                        if let Some((ref pixmap, (sx, sy), br)) = cache.shadow_pixmap {
                            if (sx - scale_x).abs() < 0.001
                                && (sy - scale_y).abs() < 0.001
                                && (br - shadow.blur_radius).abs() < 0.001
                            {
                                cached_shadow = Self::clone_pixmap(pixmap);
                            }
                        }
                    }

                    let shadow_pixmap_to_draw = if let Some(cached) = cached_shadow {
                        cached
                    } else {
                        let mut new_shadow_pixmap = Pixmap::new(sw, sh).unwrap();
                        new_shadow_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                        let shadow_render_transform =
                            tiny_skia::Transform::from_translate(pad, pad)
                                .pre_scale(scale_x, scale_y);

                        resvg::render(
                            &handle.tree,
                            shadow_render_transform,
                            &mut new_shadow_pixmap.as_mut(),
                        );

                        let color_r = shadow.color.r as u32;
                        let color_g = shadow.color.g as u32;
                        let color_b = shadow.color.b as u32;
                        let color_a = shadow.color.a as f32 / 255.0;

                        Self::recolor_shadow(
                            &mut new_shadow_pixmap,
                            color_r,
                            color_g,
                            color_b,
                            color_a,
                        );

                        if shadow.blur_radius > 0.0 {
                            Self::fast_blur(
                                new_shadow_pixmap.pixels_mut(),
                                sw as usize,
                                sh as usize,
                                shadow.blur_radius.round() as u32,
                            )?;
                        }

                        {
                            let mut cache = handle.cache.write();
                            if let Some(mut clone_to_cache) = Pixmap::new(sw, sh) {
                                clone_to_cache
                                    .data_mut()
                                    .copy_from_slice(new_shadow_pixmap.data());
                                cache.shadow_pixmap =
                                    Some((clone_to_cache, (scale_x, scale_y), shadow.blur_radius));
                            }
                        }

                        new_shadow_pixmap
                    };

                    let offset_x = x - pad + shadow.offset_x;
                    let offset_y = y - pad + shadow.offset_y;

                    let pattern = tiny_skia::Pattern::new(
                        shadow_pixmap_to_draw.as_ref(),
                        tiny_skia::SpreadMode::Pad,
                        tiny_skia::FilterQuality::Bilinear,
                        1.0,
                        tiny_skia::Transform::from_translate(offset_x, offset_y),
                    );

                    let mut paint = Paint::default();
                    paint.shader = pattern;
                    paint.anti_alias = true;

                    self.fill_rect_with_transform(
                        offset_x, offset_y, sw as f32, sh as f32, &paint, transform,
                    );
                }
            }
        }

        let mut cached_body = None;
        let tw = target_width.ceil() as u32;
        let th = target_height.ceil() as u32;

        if tw > 0 && th > 0 {
            {
                let cache = handle.cache.read();
                if let Some((ref pixmap, (sx, sy))) = cache.svg_pixmap {
                    if (sx - scale_x).abs() < 0.001 && (sy - scale_y).abs() < 0.001 {
                        cached_body = Self::clone_pixmap(pixmap);
                    }
                }
            }

            let body_pixmap_to_draw = if let Some(cached) = cached_body {
                cached
            } else {
                let mut new_body_pixmap = Pixmap::new(tw, th).unwrap();
                new_body_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                let svg_render_transform = tiny_skia::Transform::from_scale(scale_x, scale_y);
                resvg::render(
                    &handle.tree,
                    svg_render_transform,
                    &mut new_body_pixmap.as_mut(),
                );

                {
                    let mut cache = handle.cache.write();
                    if let Some(mut clone_to_cache) = Pixmap::new(tw, th) {
                        clone_to_cache
                            .data_mut()
                            .copy_from_slice(new_body_pixmap.data());
                        cache.svg_pixmap = Some((clone_to_cache, (scale_x, scale_y)));
                    }
                }

                new_body_pixmap
            };

            let final_transform = transform.pre_translate(x, y);

            let pattern = tiny_skia::Pattern::new(
                body_pixmap_to_draw.as_ref(),
                tiny_skia::SpreadMode::Pad,
                tiny_skia::FilterQuality::Bilinear,
                1.0,
                final_transform,
            );

            let mut paint = Paint::default();
            paint.shader = pattern;
            paint.anti_alias = true;

            Self::apply_opacity(&mut paint, options.and_then(|o| o.opacity));

            self.fill_rect_with_transform(x, y, tw as f32, th as f32, &paint, transform);
        }

        Ok(())
    }

    fn draw_surface(
        &mut self,
        handle: &Self::SurfaceId,
        x: f32,
        y: f32,
        width: Option<f32>,
        height: Option<f32>,
        options: Option<&SurfaceDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("draw_surface");

        let (surface_width, surface_height) = match self.surface_pixmap.get(handle.id) {
            Some(pm) => (pm.width() as f32, pm.height() as f32),
            None => return Ok(()),
        };

        let target_width = width.unwrap_or(surface_width);
        let target_height = height.unwrap_or(surface_height);

        let mut shader_transform = tiny_skia::Transform::identity();
        let scale_x = target_width / surface_width;
        let scale_y = target_height / surface_height;
        shader_transform = shader_transform
            .pre_translate(x, y)
            .pre_scale(scale_x, scale_y);

        let mut pb = tiny_skia::PathBuilder::new();
        if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, target_width, target_height) {
            pb.push_rect(rect);
        } else {
            return Ok(());
        }

        let path = if let Some(p) = pb.finish() {
            p
        } else {
            return Ok(());
        };

        let current_transform = self.transform_stack.last().copied().unwrap_or_default();

        let mut paint = Paint::default();
        paint.anti_alias = true;
        Self::apply_opacity(&mut paint, options.and_then(|o| o.opacity));

        // 克隆 pixmap 以避免借用冲突（self 上同时需要可变和不可变借用）
        let pixmap_clone = self.surface_pixmap.get(handle.id).map(|p| {
            let mut clone = Pixmap::new(p.width(), p.height()).unwrap();
            clone.data_mut().copy_from_slice(p.data());
            clone
        });

        if let Some(surface_pixmap) = pixmap_clone {
            paint.shader = tiny_skia::Pattern::new(
                surface_pixmap.as_ref(),
                tiny_skia::SpreadMode::Pad,
                tiny_skia::FilterQuality::Bicubic,
                1.0,
                shader_transform,
            );

            self.with_current_pixmap(|p, clip| {
                p.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    current_transform,
                    clip,
                );
            });
        }

        Ok(())
    }

    fn draw_text(
        &mut self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        brush: &Self::BrushHandle,
        options: Option<&TextDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("draw_text");

        let text_layout = self.create_text_layout(
            text.to_string(),
            Rect {
                x: left,
                y: top,
                w: width,
                h: height,
            },
            text_format.clone(),
        )?;

        self.draw_layout_text(&text_layout, brush, options)
    }

    fn draw_layout_text(
        &mut self,
        layout_text: &Self::LayoutText,
        brush: &Self::BrushHandle,
        options: Option<&TextDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("draw_layout_text");
        let (dpi_x, dpi_y) = self.dpi_scale;
        let bounds = layout_text.bounds();
        let buffer = layout_text.buffer();
        let text_format = layout_text.text_format();
        let left = bounds.x;
        let top = bounds.y;
        let width = bounds.w;
        let height = bounds.h;
        let phys_height = height * dpi_y;

        // 计算文本总高度
        let mut total_h = 0.0f32;
        for run in buffer.layout_runs() {
            total_h += run.line_height;
        }

        // 计算段落对齐的 Y 偏移
        let offset_y = match text_format.config.paragraph_alignment {
            ParagraphAlignment::Center if phys_height > total_h => (phys_height - total_h) / 2.0,
            ParagraphAlignment::Bottom if phys_height > total_h => phys_height - total_h,
            _ => 0.0,
        };

        // 获取渲染变换
        let transform = self.get_options_transform(options);

        let draw_glyphs = |swash_cache: &mut cosmic_text::SwashCache,
                           temp_system: &mut cosmic_text::FontSystem,
                           render_target: &mut PixmapMut,
                           clip_mask: Option<&tiny_skia::Mask>,
                           render_transform: tiny_skia::Transform,
                           override_color: Option<tiny_skia::Color>| {
            for run in buffer.layout_runs() {
                for glyph in run.glyphs.iter() {
                    let current_brush = layout_text.brush_at(glyph.metadata).unwrap_or(brush);

                    let has_rotation = render_transform.kx != 0.0 || render_transform.ky != 0.0;

                    if has_rotation {
                        let physical_glyph = glyph
                            .physical((left * dpi_x, top * dpi_y + offset_y + run.line_y), 1.0);

                        if let Some(commands) =
                            swash_cache.get_outline_commands(temp_system, physical_glyph.cache_key)
                        {
                            let gx = physical_glyph.x as f32;
                            let gy = physical_glyph.y as f32;

                            let mut pb = tiny_skia::PathBuilder::new();
                            for cmd in commands {
                                match cmd {
                                    cosmic_text::Command::MoveTo(v) => {
                                        pb.move_to(gx + v.x, gy - v.y);
                                    }
                                    cosmic_text::Command::LineTo(v) => {
                                        pb.line_to(gx + v.x, gy - v.y);
                                    }
                                    cosmic_text::Command::QuadTo(c, p) => {
                                        pb.quad_to(gx + c.x, gy - c.y, gx + p.x, gy - p.y);
                                    }
                                    cosmic_text::Command::CurveTo(c1, c2, p) => {
                                        pb.cubic_to(
                                            gx + c1.x,
                                            gy - c1.y,
                                            gx + c2.x,
                                            gy - c2.y,
                                            gx + p.x,
                                            gy - p.y,
                                        );
                                    }
                                    cosmic_text::Command::Close => {
                                        pb.close();
                                    }
                                }
                            }

                            if let Some(glyph_path) = pb.finish() {
                                let fill_color = override_color.or_else(|| {
                                    if let TinySkiaBrushHandle::Solid(c) = current_brush {
                                        Some(*c)
                                    } else {
                                        None
                                    }
                                });

                                let mut paint = Paint::default();
                                paint.anti_alias = true;

                                if let Some(c) = fill_color {
                                    paint.set_color(c);
                                } else {
                                    Self::set_brush(current_brush, &mut paint);
                                }

                                let outline_transform =
                                    render_transform.pre_scale(1.0 / dpi_x, 1.0 / dpi_y);

                                render_target.fill_path(
                                    &glyph_path,
                                    &paint,
                                    tiny_skia::FillRule::Winding,
                                    outline_transform,
                                    clip_mask,
                                );
                            }
                        }
                        continue;
                    }

                    let scale_x = (render_transform.sx * render_transform.sx
                        + render_transform.ky * render_transform.ky)
                        .sqrt();
                    let scale_y = (render_transform.kx * render_transform.kx
                        + render_transform.sy * render_transform.sy)
                        .sqrt();
                    let raster_scale = scale_x.max(scale_y).max(0.001);

                    // 使用 glyph 的字体属性创建物理字形
                    let physical_glyph = glyph.physical(
                        (
                            left * dpi_x * raster_scale,
                            (top * dpi_y + offset_y + run.line_y) * raster_scale,
                        ),
                        raster_scale,
                    );

                    // 从缓存获取字形图像
                    if let Some(image) =
                        swash_cache.get_image(temp_system, physical_glyph.cache_key)
                    {
                        let gx = physical_glyph.x as f32 + image.placement.left as f32;
                        let gy = physical_glyph.y as f32 - image.placement.top as f32;

                        let logical_gx = gx / raster_scale;
                        let logical_gy = gy / raster_scale;

                        if image.placement.width == 0 || image.placement.height == 0 {
                            continue;
                        }

                        // 创建 pixmap 并填充像素
                        if let Some(mut pixmap) =
                            Pixmap::new(image.placement.width, image.placement.height)
                        {
                            let pixels = pixmap.pixels_mut();
                            let mut i = 0;

                            // 确定基础字形颜色
                            let mut base_glyph_color = override_color;
                            if base_glyph_color.is_none() {
                                if let TinySkiaBrushHandle::Solid(c) = current_brush {
                                    let color_op = *c;
                                    base_glyph_color = Some(color_op);
                                }
                            }

                            // 根据图像内容类型处理像素
                            match image.content {
                                cosmic_text::SwashContent::SubpixelMask => {
                                    for _y in 0..image.placement.height {
                                        for _x in 0..image.placement.width {
                                            let r = image.data[i * 3];
                                            let g = image.data[i * 3 + 1];
                                            let b = image.data[i * 3 + 2];
                                            let a = ((r as u32 + g as u32 + b as u32) / 3) as f32
                                                / 255.0;
                                            pixels[i] = Self::get_glyph_pixel(
                                                base_glyph_color,
                                                a,
                                                (a * 255.0).round() as u8,
                                            );
                                            i += 1;
                                        }
                                    }
                                }
                                cosmic_text::SwashContent::Mask => {
                                    for _y in 0..image.placement.height {
                                        for _x in 0..image.placement.width {
                                            let a = image.data[i] as f32 / 255.0;
                                            pixels[i] = Self::get_glyph_pixel(
                                                base_glyph_color,
                                                a,
                                                image.data[i],
                                            );
                                            i += 1;
                                        }
                                    }
                                }
                                cosmic_text::SwashContent::Color => {
                                    for _y in 0..image.placement.height {
                                        for _x in 0..image.placement.width {
                                            let a = image.data[i * 4 + 3];
                                            if let Some(c) = base_glyph_color {
                                                pixels[i] = Self::get_glyph_pixel(
                                                    Some(c),
                                                    a as f32 / 255.0,
                                                    a,
                                                );
                                            } else {
                                                let r = image.data[i * 4];
                                                let g = image.data[i * 4 + 1];
                                                let b = image.data[i * 4 + 2];
                                                let color =
                                                    tiny_skia::Color::from_rgba8(r, g, b, a);
                                                pixels[i] = color.premultiply().to_color_u8();
                                            }
                                            i += 1;
                                        }
                                    }
                                }
                            }

                            let inv_raster = 1.0 / raster_scale;

                            // 处理渐变画笔
                            if base_glyph_color.is_none() {
                                let mut stops = Vec::new();
                                let mut local_shader = None;

                                let shader_transform = render_transform
                                    .pre_translate(logical_gx, logical_gy)
                                    .pre_scale(inv_raster, inv_raster)
                                    .invert()
                                    .unwrap_or(tiny_skia::Transform::identity());

                                if let TinySkiaBrushHandle::Gradient { gradient } = current_brush {
                                    match gradient {
                                        Gradient::Linear { start, end, colors } => {
                                            for (pos, c) in colors {
                                                stops.push(tiny_skia::GradientStop::new(
                                                    *pos,
                                                    c.to_tiny_skia(),
                                                ));
                                            }
                                            local_shader = tiny_skia::LinearGradient::new(
                                                tiny_skia::Point::from_xy(start.0, start.1),
                                                tiny_skia::Point::from_xy(end.0, end.1),
                                                stops,
                                                tiny_skia::SpreadMode::Pad,
                                                shader_transform,
                                            );
                                        }
                                        Gradient::Radial {
                                            center,
                                            radius,
                                            colors,
                                        } => {
                                            for (pos, c) in colors {
                                                stops.push(tiny_skia::GradientStop::new(
                                                    *pos,
                                                    c.to_tiny_skia(),
                                                ));
                                            }
                                            local_shader = tiny_skia::RadialGradient::new(
                                                tiny_skia::Point::from_xy(center.0, center.1),
                                                0.0,
                                                tiny_skia::Point::from_xy(center.0, center.1),
                                                *radius,
                                                stops,
                                                tiny_skia::SpreadMode::Pad,
                                                shader_transform,
                                            );
                                        }
                                    }
                                }

                                if let Some(shader) = local_shader {
                                    let mut mask_paint = Paint::default();
                                    mask_paint.shader = shader;
                                    mask_paint.blend_mode = tiny_skia::BlendMode::SourceIn;
                                    mask_paint.anti_alias = true;

                                    if let Some(rect) = tiny_skia::Rect::from_xywh(
                                        0.0,
                                        0.0,
                                        image.placement.width as f32,
                                        image.placement.height as f32,
                                    ) {
                                        pixmap.fill_rect(
                                            rect,
                                            &mask_paint,
                                            tiny_skia::Transform::identity(),
                                            None,
                                        );
                                    }
                                }
                            }

                            // 计算局部变换
                            let local_transform = render_transform
                                .pre_translate(logical_gx, logical_gy)
                                .pre_scale(inv_raster, inv_raster);

                            let mut pixmap_paint = PixmapPaint::default();
                            pixmap_paint.quality = tiny_skia::FilterQuality::Bicubic;

                            // 绘制 pixmap 到目标
                            render_target.draw_pixmap(
                                0,
                                0,
                                pixmap.as_ref(),
                                &pixmap_paint,
                                local_transform,
                                clip_mask,
                            );
                        }
                    }
                }
            }
        };

        // ====================================================================
        // 第三阶段：绘制阴影（如果有）
        // ====================================================================
        let mut font_system = FONT_SYSTEM.lock();
        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let shadow_color = shadow.color.to_tiny_skia();

                // ------------------------------------------------------------
                // 子阶段 3.1：硬阴影（无模糊）
                // ------------------------------------------------------------
                if shadow.blur_radius <= 0.0 {
                    let shadow_transform =
                        transform.post_translate(shadow.offset_x, shadow.offset_y);
                    let (cache, mut p, c) = self.split_pixmap_and_cache();
                    draw_glyphs(
                        cache,
                        &mut font_system,
                        &mut p,
                        c,
                        shadow_transform,
                        Some(shadow_color),
                    );
                }
                // ------------------------------------------------------------
                // 子阶段 3.2：软阴影（有模糊）
                // ------------------------------------------------------------
                else {
                    // 步骤 1：从变换中移除旋转（用于离屏渲染）
                    let base_transform = self.get_current_transform();

                    let pad = (shadow.blur_radius.ceil() * 3.0).max(0.0);
                    let pad_log = pad;

                    // 计算文本边界框
                    let mut bb_min_x = f32::MAX;
                    let mut bb_min_y = f32::MAX;
                    let mut bb_max_x = f32::MIN;
                    let mut bb_max_y = f32::MIN;

                    for run in buffer.layout_runs() {
                        for glyph in run.glyphs.iter() {
                            let px = left + glyph.x / dpi_x;
                            let py = top + offset_y / dpi_y + run.line_y / dpi_y;
                            bb_min_x = bb_min_x.min(px);
                            bb_min_y = bb_min_y.min(py - run.line_height / dpi_y);
                            bb_max_x = bb_max_x.max(px + glyph.w / dpi_x);
                            bb_max_y = bb_max_y.max(py + run.line_height / dpi_y);
                        }
                    }
                    if bb_min_x > bb_max_x {
                        bb_min_x = left;
                        bb_max_x = left + width;
                        bb_min_y = top;
                        bb_max_y = top + height;
                    }

                    let sw = ((bb_max_x - bb_min_x + pad_log * 2.0) * dpi_x).ceil() as u32;
                    let sh = ((bb_max_y - bb_min_y + pad_log * 2.0) * dpi_y).ceil() as u32;

                    if sw > 0 && sh > 0 && sw < 8192 && sh < 8192 {
                        if let Some(mut shadow_pixmap) = Pixmap::new(sw, sh) {
                            shadow_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                            // 步骤 2：将字形绘制到阴影 pixmap（无旋转）
                            let bounds_transform = base_transform
                                .pre_translate(-bb_min_x + pad_log, -bb_min_y + pad_log);
                            let swash_cache = &mut self.swash_cache;
                            draw_glyphs(
                                swash_cache,
                                &mut font_system,
                                &mut shadow_pixmap.as_mut(),
                                None,
                                bounds_transform,
                                Some(shadow_color),
                            );

                            // 步骤 3：模糊处理
                            let blur_px =
                                (shadow.blur_radius * 2.0 * dpi_x).round().max(1.0) as u32;
                            Self::fast_blur(
                                shadow_pixmap.pixels_mut(),
                                sw as usize,
                                sh as usize,
                                blur_px,
                            )
                            .ok();

                            // 步骤 4：将模糊后的阴影合成到屏幕上
                            let user_xform = if transform != base_transform {
                                base_transform
                                    .invert()
                                    .map(|inv| inv.pre_concat(transform))
                                    .unwrap_or(tiny_skia::Transform::identity())
                            } else {
                                tiny_skia::Transform::identity()
                            };

                            let shadow_draw_transform = user_xform.pre_translate(
                                bb_min_x - pad_log + shadow.offset_x,
                                bb_min_y - pad_log + shadow.offset_y,
                            );

                            self.with_current_pixmap(|p, c| {
                                let mut shadow_paint = PixmapPaint::default();
                                shadow_paint.quality = tiny_skia::FilterQuality::Bicubic;
                                p.draw_pixmap(
                                    0,
                                    0,
                                    shadow_pixmap.as_ref(),
                                    &shadow_paint,
                                    shadow_draw_transform,
                                    c,
                                );
                            });
                        }
                    }
                }
            }
        }

        let (cache, mut p, c) = self.split_pixmap_and_cache();
        draw_glyphs(cache, &mut font_system, &mut p, c, transform, None);

        Ok(())
    }

    fn draw_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        stroke_width: f32,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("draw_path");
        let ts_path = if let Some(p) = Self::build_tiny_skia_path(path) {
            p
        } else {
            return Ok(());
        };

        let mut paint = Paint::default();
        paint.anti_alias = true;
        Self::set_brush(brush, &mut paint);

        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = stroke_width;

        let transform = self.get_options_transform(options);

        self.with_current_pixmap(|p, clip| {
            p.stroke_path(&ts_path, &paint, &stroke, transform, clip);
        });
        Ok(())
    }

    fn fill_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("fill_path");
        let ts_path = if let Some(p) = Self::build_tiny_skia_path(path) {
            p
        } else {
            return Ok(());
        };

        let mut paint = Paint::default();
        paint.anti_alias = true;

        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let _ = self.draw_shadow(path, shadow, opts.transform.as_ref());
            }
        }

        Self::set_brush(brush, &mut paint);

        let transform = self.get_options_transform(options);

        self.with_current_pixmap(|p, clip| {
            p.fill_path(
                &ts_path,
                &paint,
                tiny_skia::FillRule::Winding,
                transform,
                clip,
            )
        });
        Ok(())
    }

    fn draw_quad(
        &mut self,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        border_width: f32,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("draw_quad");
        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = border_width;

        let transform = self.get_options_transform(options);

        let mut paint = Paint::default();
        paint.anti_alias = true;

        Self::set_brush(brush, &mut paint);

        let path = Path::from_rect(left, top, width, height);
        if let Some(ts_path) = Self::build_tiny_skia_path(&path) {
            self.with_current_pixmap(|p, clip| {
                p.stroke_path(&ts_path, &paint, &stroke, transform, clip)
            });
        }
        Ok(())
    }

    fn fill_quad(
        &mut self,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        brush: &Self::BrushHandle,
        corner_radius: Option<f32>,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("fill_quad");
        let transform = self.get_options_transform(options);

        let mut paint = Paint::default();
        paint.anti_alias = true;
        Self::set_brush(brush, &mut paint);

        let r = corner_radius.unwrap_or(0.0);

        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let path = Path::from_rounded_rect(left, top, width, height, r);
                let _ = self.draw_shadow(&path, shadow, opts.transform.as_ref());
            }
        }

        if r <= 0.0 {
            if let Some(rect) = tiny_skia::Rect::from_xywh(left, top, width, height) {
                self.with_current_pixmap(|p, clip| p.fill_rect(rect, &paint, transform, clip));
            }
        } else {
            let path = Path::from_rounded_rect(left, top, width, height, r);
            if let Some(ts_path) = Self::build_tiny_skia_path(&path) {
                self.with_current_pixmap(|p, clip| {
                    p.fill_path(
                        &ts_path,
                        &paint,
                        tiny_skia::FillRule::Winding,
                        transform,
                        clip,
                    )
                });
            }
        }

        Ok(())
    }

    fn blur_quad(
        &mut self,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        corner_radius: f32,
        blur_radius: f32,
        transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("blur_quad");
        let path = Path::from_rounded_rect(left, top, width, height, corner_radius);
        self.blur_path(&path, blur_radius, transform)
    }

    fn blur_path(
        &mut self,
        path: &Path,
        blur_radius: f32,
        options_transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("blur_path");
        let ts_path = if let Some(p) = Self::build_tiny_skia_path(path) {
            p
        } else {
            return Ok(());
        };

        if blur_radius <= 0.0 {
            return Ok(());
        }

        let transform = if let Some(t) = options_transform {
            self.get_current_transform().pre_concat(t.to_tiny_skia())
        } else {
            self.get_current_transform()
        };

        let screen_path = if let Some(p) = ts_path.transform(transform) {
            p
        } else {
            return Ok(());
        };

        let bounds = screen_path.bounds();
        let pad = blur_radius.ceil() * 2.5;

        let left = (bounds.left() - pad).floor() as i32;
        let top = (bounds.top() - pad).floor() as i32;
        let right = (bounds.right() + pad).ceil() as i32;
        let bottom = (bounds.bottom() + pad).ceil() as i32;

        let mut extracted_pixels: Option<Pixmap> = None;
        let mut actual_left = 0.0;
        let mut actual_top = 0.0;

        let w = (right - left) as u32;
        let h = (bottom - top) as u32;

        if w > 0 && h > 0 {
            if let Some(mut pixmap) = Pixmap::new(w, h) {
                pixmap.fill(tiny_skia::Color::TRANSPARENT);

                self.with_current_pixmap(|p, _clip| {
                    let p_width = p.width() as i32;
                    let p_height = p.height() as i32;

                    let c_left = left.clamp(0, p_width);
                    let c_top = top.clamp(0, p_height);
                    let c_right = right.clamp(0, p_width);
                    let c_bottom = bottom.clamp(0, p_height);

                    let copy_w = (c_right - c_left) as usize;
                    let copy_h = (c_bottom - c_top) as usize;

                    if copy_w > 0 && copy_h > 0 {
                        let src_pixels = p.pixels_mut();
                        let dst_pixels = pixmap.pixels_mut();

                        let src_stride = p_width as usize;
                        let dst_stride = w as usize;

                        let src_start_x = c_left as usize;
                        let dst_start_x = (c_left - left) as usize;
                        let dst_start_y = (c_top - top) as usize;

                        for y in 0..copy_h {
                            let src_y = c_top as usize + y;
                            let dst_y = dst_start_y + y;

                            let src_offset = src_y * src_stride + src_start_x;
                            let dst_offset = dst_y * dst_stride + dst_start_x;

                            dst_pixels[dst_offset..dst_offset + copy_w]
                                .copy_from_slice(&src_pixels[src_offset..src_offset + copy_w]);
                        }
                    }
                });

                extracted_pixels = Some(pixmap);
                actual_left = left as f32;
                actual_top = top as f32;
            }
        }

        if let Some(mut blur_pixmap) = extracted_pixels {
            let bw = blur_pixmap.width() as usize;
            let bh = blur_pixmap.height() as usize;
            Self::fast_blur(blur_pixmap.pixels_mut(), bw, bh, blur_radius.round() as u32)?;

            let mut pattern_paint = Paint::default();
            pattern_paint.anti_alias = true;

            let pattern = tiny_skia::Pattern::new(
                blur_pixmap.as_ref(),
                tiny_skia::SpreadMode::Pad,
                tiny_skia::FilterQuality::Bilinear,
                1.0,
                tiny_skia::Transform::from_translate(actual_left, actual_top),
            );

            pattern_paint.shader = pattern;

            self.with_current_pixmap(|p, clip| {
                p.fill_path(
                    &screen_path,
                    &pattern_paint,
                    tiny_skia::FillRule::Winding,
                    tiny_skia::Transform::identity(),
                    clip,
                );
            });
        }

        Ok(())
    }

    fn push_clip(&mut self, rect: (f32, f32, f32, f32)) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("push_clip");
        let path = Path::from_rect(rect.0, rect.1, rect.2, rect.3);
        if let Some(ts_path) = Self::build_tiny_skia_path(&path) {
            self.push_path_internal(ts_path);
        }
        Ok(())
    }

    fn push_rounded_clip(
        &mut self,
        rect: (f32, f32, f32, f32),
        radius: f32,
    ) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("push_rounded_clip");
        let path = Path::from_rounded_rect(rect.0, rect.1, rect.2, rect.3, radius);
        if let Some(ts_path) = Self::build_tiny_skia_path(&path) {
            self.push_path_internal(ts_path);
        }
        Ok(())
    }

    fn push_path_clip(&mut self, path: &Path) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("push_path_clip");
        if let Some(ts_path) = Self::build_tiny_skia_path(path) {
            self.push_path_internal(ts_path);
        }
        Ok(())
    }

    fn pop_clip(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("pop_clip");
        if let Some(target) = target_depth {
            let target = target as usize;
            if self.clip_stack.len() > target {
                self.clip_stack.truncate(target);
            }
        } else {
            if !self.clip_stack.is_empty() {
                self.clip_stack.pop();
            }
        }
        self.update_clip_mask();
        Ok(())
    }

    fn get_clip_depth(&mut self) -> Result<u32, Self::Error> {
        let _timer = FuncTimer::new("get_clip_depth");
        Ok(self.clip_stack.len() as u32)
    }

    fn suspend_clip(&mut self) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("suspend_clip");
        self.clip_suspended = true;
        self.update_clip_mask();
        Ok(())
    }

    fn resume_clip(&mut self) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("resume_clip");
        self.clip_suspended = false;
        self.update_clip_mask();
        Ok(())
    }

    fn push_transform(&mut self, transform: &Transform2D) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("push_transform");
        let ts = transform.to_tiny_skia();
        let current = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(tiny_skia::Transform::identity());
        self.transform_stack.push(current.pre_concat(ts));
        Ok(())
    }

    fn pop_transform(&mut self, _target_depth: Option<u32>) -> Result<(), Self::Error> {
        let _timer = FuncTimer::new("pop_transform");
        if let Some(target) = _target_depth {
            let target = target as usize;
            if self.transform_stack.len() > target && target > 0 {
                self.transform_stack.truncate(target);
            }
        } else {
            if self.transform_stack.len() > 1 {
                self.transform_stack.pop();
            }
        }
        Ok(())
    }

    fn get_transform_depth(&mut self) -> Result<u32, Self::Error> {
        let _timer = FuncTimer::new("get_transform_depth");
        Ok(self.transform_stack.len() as u32)
    }

    fn capture_snapshot(
        &mut self,
        _rect: Option<(f32, f32, u32, u32)>,
    ) -> Result<Vec<u8>, Self::Error> {
        let _timer = FuncTimer::new("capture_snapshot");
        let data = self.default_pixmap.data();
        let mut out = Vec::with_capacity(data.len());
        for chunk in data.chunks_exact(4) {
            let pr = chunk[0] as u32;
            let pg = chunk[1] as u32;
            let pb = chunk[2] as u32;
            let pa = chunk[3] as u32;

            if pa == 0 {
                out.push(0);
                out.push(0);
                out.push(0);
                out.push(0);
            } else {
                let r = (pr * 255 + pa / 2) / pa;
                let g = (pg * 255 + pa / 2) / pa;
                let b = (pb * 255 + pa / 2) / pa;
                out.push(r.min(255) as u8);
                out.push(g.min(255) as u8);
                out.push(b.min(255) as u8);
                out.push(pa as u8);
            }
        }
        Ok(out)
    }
}
