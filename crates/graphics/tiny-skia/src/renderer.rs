use crate::error::TinySkiaError;
use crate::handle::{
    SurfaceSlotId, TinySkiaBrushHandle, TinySkiaImageHandle, TinySkiaSurfaceId,
    TinySkiaTextFormatHandle,
};
use flor_base::graphics::{
    Gradient, HitTestResult, ImageDrawOptions, ParagraphAlignment, Path, PathDrawOptions, Render,
    RenderContext, TextDrawOptions,
};
use flor_base::types::{Color, Transform2D};
use slotmap::SlotMap;
use tiny_skia::Pixmap;

#[cfg(feature = "svg")]
use {
    crate::handle::{SvgSlotId, TinySkiaSvgHandle},
    flor_base::graphics::SvgDrawOptions,
};

mod config;
use crate::display_context::{DisplayContext, NativeDisplayContext};
use crate::to_tiny_skia::ToTinySkia;
pub use config::*;

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
    pub font_system: cosmic_text::FontSystem,
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
            font_system: cosmic_text::FontSystem::new(),
            swash_cache: cosmic_text::SwashCache::new(),
            clip_stack: Vec::new(),
            active_clip: None,
            clip_suspended: false,
            dpi_scale: (1.0, 1.0),
        })
    }
}

impl TinySkiaRenderer {
    fn get_current_transform(&self) -> tiny_skia::Transform {
        self.transform_stack
            .last()
            .copied()
            .unwrap_or(tiny_skia::Transform::identity())
    }

    /// Run a closure with the current render target PixmapMut and the active ClipMask
    fn with_current_pixmap<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut tiny_skia::PixmapMut, Option<&tiny_skia::Mask>) -> R,
    {
        let clip = self.active_clip.as_ref();
        if let Some(target) = self.current_render_target {
            if let Some(surface) = self.surface_pixmap.get_mut(target) {
                return f(&mut surface.as_mut(), clip);
            }
        }
        f(&mut self.default_pixmap.as_mut(), clip)
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
        shadow: &flor_base::graphics::Shadow,
        options_transform: Option<&Transform2D>,
    ) -> Result<(), crate::error::TinySkiaError> {
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
        // Apply offset
        let transform = transform.post_translate(shadow.offset_x, shadow.offset_y);

        let mut paint = tiny_skia::Paint::default();
        paint.anti_alias = true;
        paint.set_color(shadow.color.to_tiny_skia());

        let bounds = ts_path.bounds();
        let pad = shadow.blur_radius.ceil() as f32 * 2.0;

        let left = (bounds.left() - pad).floor();
        let top = (bounds.top() - pad).floor();
        let right = (bounds.right() + pad).ceil();
        let bottom = (bounds.bottom() + pad).ceil();

        let width = (right - left) as u32;
        let height = (bottom - top) as u32;

        if width == 0 || height == 0 {
            return Ok(());
        }

        if let Some(mut blur_pixmap) = tiny_skia::Pixmap::new(width, height) {
            let mut local_transform = transform;
            local_transform = local_transform.post_translate(-left, -top);

            blur_pixmap.fill_path(
                &ts_path,
                &paint,
                tiny_skia::FillRule::Winding,
                local_transform,
                None,
            );

            crate::fast_box_blur::fast_box_blur(
                blur_pixmap.pixels_mut(),
                width as usize,
                height as usize,
                shadow.blur_radius.round() as u32,
            );

            let mut draw_transform = tiny_skia::Transform::identity();
            draw_transform = draw_transform.post_translate(left, top);

            self.with_current_pixmap(|p, clip| {
                p.draw_pixmap(
                    0,
                    0,
                    blur_pixmap.as_ref(),
                    &tiny_skia::PixmapPaint::default(),
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
                new_mask.clone_from(prev); // `tiny_skia::Mask` leverages `clone_from` for efficient buffer copying
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
}

impl RenderContext for TinySkiaRenderer {
    type Error = TinySkiaError;
    type ImageHandle = TinySkiaImageHandle;
    type SurfaceId = TinySkiaSurfaceId;
    type BrushHandle = TinySkiaBrushHandle;
    #[cfg(feature = "svg")]
    type SvgHandle = TinySkiaSvgHandle;
    type TextFormatHandle = TinySkiaTextFormatHandle;

    fn begin(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end(&mut self) -> Result<(), Self::Error> {
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
        self.with_current_pixmap(|p, clip| {
            if let Some(c) = clip {
                let mut paint = tiny_skia::Paint::default();
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
        Ok(())
    }

    fn update_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        if let Some(pixmap) = Pixmap::new(width.max(1), height.max(1)) {
            self.default_pixmap = pixmap;
            self.clip_stack.clear();
            self.active_clip = None;
        }
        Ok(())
    }

    fn set_scale_factor(&mut self, dpi_x: f32, dpi_y: f32) -> Result<(), Self::Error> {
        self.dpi_scale = (dpi_x, dpi_y);
        if let Some(first) = self.transform_stack.first_mut() {
            *first = tiny_skia::Transform::from_scale(dpi_x, dpi_y);
        }
        Ok(())
    }

    fn create_surface(&mut self, width: u32, height: u32) -> Result<Self::SurfaceId, Self::Error> {
        let pixmap = tiny_skia::Pixmap::new(width.max(1), height.max(1))
            .ok_or(TinySkiaError::CreateSurfaceError)?;
        let id = self.surface_pixmap.insert(pixmap);
        Ok(TinySkiaSurfaceId { id })
    }

    fn set_render_target(&mut self, surface_id: &Self::SurfaceId) -> Result<(), Self::Error> {
        if self.surface_pixmap.contains_key(surface_id.id) {
            self.current_render_target = Some(surface_id.id);
            Ok(())
        } else {
            Err(TinySkiaError::SurfaceNotFoundError)
        }
    }

    fn reset_render_target(&mut self) -> Result<(), Self::Error> {
        self.current_render_target = None;
        Ok(())
    }

    fn create_image_from_bytes(&mut self, bytes: &[u8]) -> Result<Self::ImageHandle, Self::Error> {
        let img = image::load_from_memory(bytes).map_err(|_| TinySkiaError::ImageDecodeError)?;
        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        let mut pixmap =
            tiny_skia::Pixmap::new(width, height).ok_or(TinySkiaError::CreateSurfaceError)?;

        let slice = pixmap.data_mut();
        // image crate rgba is R, G, B, A order (unpremultiplied typically)
        // tiny-skia requires premultiplied R, G, B, A or B, G, R, A depending on platform, but color::from_rgba assumes standard unpremultiplied
        // The most proper way is to use Pixmap::data_mut() and write [u8; 4] array if memory layout matches, or loop through pixels.
        for (src, dst) in rgba.chunks_exact(4).zip(slice.chunks_exact_mut(4)) {
            let r = src[0];
            let g = src[1];
            let b = src[2];
            let a = src[3];
            // tiny-skia expects premultiplied alpha
            let color = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
            dst[0] = color.red();
            dst[1] = color.green();
            dst[2] = color.blue();
            dst[3] = color.alpha();
        }

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
        let mut frames = Vec::with_capacity(raw_bytes.len());
        for frame_bytes in raw_bytes {
            let mut pixmap =
                tiny_skia::Pixmap::new(width, height).ok_or(TinySkiaError::CreateSurfaceError)?;
            let slice = pixmap.data_mut();

            for (src, dst) in frame_bytes.chunks_exact(4).zip(slice.chunks_exact_mut(4)) {
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
        let mut handle = TinySkiaTextFormatHandle::default();
        handle.font_family_name = font_family_name.to_string();
        Ok(handle)
    }

    #[cfg(feature = "memory-font")]
    fn create_text_format_from_bytes(
        &mut self,
        font_data: &[u8],
        _ttc_index: u32,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        let source = cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(font_data.to_vec()));
        self.font_system.db_mut().load_font_source(source);
        let handle = TinySkiaTextFormatHandle::default();
        Ok(handle)
    }

    fn measure_text(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
    ) -> Result<(f32, f32), Self::Error> {
        let (dpi_x, dpi_y) = self.dpi_scale;
        let font_size = text_format.font_size * dpi_y;
        let mut temp_system = cosmic_text::FontSystem::new();
        let mut buffer = cosmic_text::Buffer::new(
            &mut temp_system,
            cosmic_text::Metrics::new(font_size, font_size * text_format.line_height_factor),
        );
        buffer.set_size(
            &mut temp_system,
            if width > 0.0 {
                Some(width * dpi_x)
            } else {
                None
            },
            if height > 0.0 {
                Some(height * dpi_y)
            } else {
                None
            },
        );
        buffer.set_wrap(&mut temp_system, text_format.to_cosmic_wrap());
        buffer.set_text(
            &mut temp_system,
            text,
            &text_format.to_cosmic_attrs(),
            cosmic_text::Shaping::Advanced,
            text_format.to_cosmic_align(),
        );
        buffer.shape_until_scroll(&mut temp_system, false);

        let mut max_w = 0.0f32;
        let mut total_h = 0.0f32;
        for run in buffer.layout_runs() {
            max_w = max_w.max(run.line_w);
            total_h += run.line_height;
        }

        Ok((max_w / dpi_x, total_h / dpi_y))
    }

    fn hit_test_point(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
        x: f32,
        y: f32,
    ) -> Result<HitTestResult, Self::Error> {
        let (dpi_x, dpi_y) = self.dpi_scale;
        let font_size = text_format.font_size * dpi_y;
        let phys_width = width * dpi_x;
        let phys_height = height * dpi_y;

        let mut temp_system = cosmic_text::FontSystem::new();
        let mut buffer = cosmic_text::Buffer::new(
            &mut temp_system,
            cosmic_text::Metrics::new(font_size, font_size * text_format.line_height_factor),
        );
        buffer.set_size(
            &mut temp_system,
            if width > 0.0 { Some(phys_width) } else { None },
            if height > 0.0 {
                Some(phys_height)
            } else {
                None
            },
        );
        buffer.set_wrap(&mut temp_system, text_format.to_cosmic_wrap());
        buffer.set_text(
            &mut temp_system,
            text,
            &text_format.to_cosmic_attrs(),
            cosmic_text::Shaping::Advanced,
            text_format.to_cosmic_align(),
        );
        buffer.shape_until_scroll(&mut temp_system, false);

        let mut total_h = 0.0f32;
        for run in buffer.layout_runs() {
            total_h += run.line_height;
        }

        let offset_y = match text_format.paragraph_alignment {
            ParagraphAlignment::Center if phys_height > total_h => (phys_height - total_h) / 2.0,
            ParagraphAlignment::Bottom if phys_height > total_h => phys_height - total_h,
            _ => 0.0,
        };

        if let Some(cursor) = buffer.hit(x * dpi_x, y * dpi_y - offset_y) {
            Ok(HitTestResult {
                is_inside: true,
                text_index: cursor.index,
                is_trailing: false,
                is_trimmed: false,
                rect: (0.0, 0.0, 0.0, 0.0),
            })
        } else {
            Ok(HitTestResult {
                is_inside: false,
                text_index: text.len(),
                is_trailing: true,
                is_trimmed: false,
                rect: (0.0, 0.0, 0.0, 0.0),
            })
        }
    }

    fn hit_test_text_position(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
        text_index: usize,
        trailing: bool,
    ) -> Result<(f32, f32), Self::Error> {
        let (dpi_x, dpi_y) = self.dpi_scale;
        let font_size = text_format.font_size * dpi_y;
        let phys_width = width * dpi_x;
        let phys_height = height * dpi_y;

        let mut temp_system = cosmic_text::FontSystem::new();
        let mut buffer = cosmic_text::Buffer::new(
            &mut temp_system,
            cosmic_text::Metrics::new(font_size, font_size * text_format.line_height_factor),
        );
        buffer.set_size(
            &mut temp_system,
            if width > 0.0 { Some(phys_width) } else { None },
            if height > 0.0 {
                Some(phys_height)
            } else {
                None
            },
        );
        buffer.set_wrap(&mut temp_system, text_format.to_cosmic_wrap());
        buffer.set_text(
            &mut temp_system,
            text,
            &text_format.to_cosmic_attrs(),
            cosmic_text::Shaping::Advanced,
            text_format.to_cosmic_align(),
        );
        buffer.shape_until_scroll(&mut temp_system, false);

        let mut total_h = 0.0f32;
        for run in buffer.layout_runs() {
            total_h += run.line_height;
        }

        let offset_y = match text_format.paragraph_alignment {
            ParagraphAlignment::Center if phys_height > total_h => (phys_height - total_h) / 2.0,
            ParagraphAlignment::Bottom if phys_height > total_h => phys_height - total_h,
            _ => 0.0,
        };

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                if glyph.start <= text_index && glyph.end > text_index {
                    let mut x = glyph.x;
                    if trailing {
                        x += glyph.w;
                    }
                    return Ok((
                        x / dpi_x,
                        (run.line_y - run.line_height + glyph.y + offset_y) / dpi_y,
                    ));
                }
            }
        }

        Ok((0.0, offset_y / dpi_y))
    }

    fn create_solid_color_brush(
        &mut self,
        color: Color,
        opacity: Option<f32>,
    ) -> Result<Self::BrushHandle, Self::Error> {
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
        let target_width = width.unwrap_or(handle.width as f32);
        let target_height = height.unwrap_or(handle.height as f32);

        let mut paint = tiny_skia::Paint::default();
        paint.anti_alias = true;

        let mut shader_transform = tiny_skia::Transform::identity();
        let scale_x = target_width / handle.width as f32;
        let scale_y = target_height / handle.height as f32;
        shader_transform = shader_transform
            .pre_translate(x, y)
            .pre_scale(scale_x, scale_y);

        if let Some(opts) = options {
            if let Some(op) = opts.opacity {
                paint.set_color_rgba8(255, 255, 255, (op * 255.0).clamp(0.0, 255.0) as u8);
            }
        }

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

        let transform = if let Some(opts) = options {
            if let Some(t) = &opts.transform {
                self.get_current_transform().pre_concat(t.to_tiny_skia())
            } else {
                self.get_current_transform()
            }
        } else {
            self.get_current_transform()
        };

        let frame_index = options.and_then(|o| o.frame_index).unwrap_or(0);
        let frames = &handle.frames;
        let pixmap = frames.get(frame_index).unwrap_or_else(|| &frames[0]);

        // 1. Draw shadow if present
        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let pad = (shadow.blur_radius.ceil() * 2.5).max(0.0);
                let sw = (target_width + pad * 2.0).ceil() as u32;
                let sh = (target_height + pad * 2.0).ceil() as u32;

                if sw > 0 && sh > 0 {
                    if let Some(mut shadow_pixmap) = tiny_skia::Pixmap::new(sw, sh) {
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

                        let mut img_paint = tiny_skia::Paint::default();
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

                        for pixel in shadow_pixmap.pixels_mut() {
                            let a = pixel.alpha() as f32 / 255.0;
                            let final_a = a * color_a;
                            if final_a > 0.0 {
                                let fa8 = (final_a * 255.0).round() as u8;
                                let fr8 = (color_r as f32 * final_a).round() as u8;
                                let fg8 = (color_g as f32 * final_a).round() as u8;
                                let fb8 = (color_b as f32 * final_a).round() as u8;

                                *pixel =
                                    tiny_skia::PremultipliedColorU8::from_rgba(fr8, fg8, fb8, fa8)
                                        .unwrap_or(tiny_skia::PremultipliedColorU8::TRANSPARENT);
                            } else {
                                *pixel = tiny_skia::PremultipliedColorU8::TRANSPARENT;
                            }
                        }

                        if shadow.blur_radius > 0.0 {
                            crate::fast_box_blur::fast_box_blur(
                                shadow_pixmap.pixels_mut(),
                                sw as usize,
                                sh as usize,
                                shadow.blur_radius.round() as u32,
                            );
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

                        let mut paint = tiny_skia::Paint::default();
                        paint.shader = pattern;
                        paint.anti_alias = true;

                        let mut pb2 = tiny_skia::PathBuilder::new();
                        if let Some(rect) =
                            tiny_skia::Rect::from_xywh(offset_x, offset_y, sw as f32, sh as f32)
                        {
                            pb2.push_rect(rect);
                            if let Some(path2) = pb2.finish() {
                                self.with_current_pixmap(|p, clip| {
                                    p.fill_path(
                                        &path2,
                                        &paint,
                                        tiny_skia::FillRule::Winding,
                                        transform,
                                        clip,
                                    );
                                });
                            }
                        }
                    }
                }
            }
        }

        let transform = if let Some(opts) = options {
            if let Some(t) = &opts.transform {
                self.get_current_transform().pre_concat(t.to_tiny_skia())
            } else {
                self.get_current_transform()
            }
        } else {
            self.get_current_transform()
        };

        let frame_index = options.and_then(|o| o.frame_index).unwrap_or(0);
        let frames = &handle.frames;
        let pixmap = frames.get(frame_index).unwrap_or_else(|| &frames[0]);

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

        // 1. Draw shadow if present
        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let pad = (shadow.blur_radius.ceil() * 2.5).max(0.0);

                let sw = (target_width + pad * 2.0).ceil() as u32;
                let sh = (target_height + pad * 2.0).ceil() as u32;

                if sw > 0 && sh > 0 {
                    let mut cached_shadow = None;

                    // Scope for reading lock
                    {
                        let cache = handle.cache.read();
                        if let Some((ref pixmap, (sx, sy), br)) = cache.shadow_pixmap {
                            // Check if scale and blur_radius are approximately the same
                            if (sx - scale_x).abs() < 0.001
                                && (sy - scale_y).abs() < 0.001
                                && (br - shadow.blur_radius).abs() < 0.001
                            {
                                // Make a clone of the pixmap to use outside the lock
                                if let Some(mut clone_pixmap) =
                                    tiny_skia::Pixmap::new(pixmap.width(), pixmap.height())
                                {
                                    clone_pixmap.data_mut().copy_from_slice(pixmap.data());
                                    cached_shadow = Some(clone_pixmap);
                                }
                            }
                        }
                    }

                    let shadow_pixmap_to_draw = if let Some(cached) = cached_shadow {
                        cached
                    } else {
                        // Needs rendering
                        let mut new_shadow_pixmap = tiny_skia::Pixmap::new(sw, sh).unwrap(); // fallback safe
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

                        for pixel in new_shadow_pixmap.pixels_mut() {
                            let a = pixel.alpha() as f32 / 255.0;
                            let final_a = a * color_a;
                            if final_a > 0.0 {
                                let fa8 = (final_a * 255.0).round() as u8;
                                let fr8 = (color_r as f32 * final_a).round() as u8;
                                let fg8 = (color_g as f32 * final_a).round() as u8;
                                let fb8 = (color_b as f32 * final_a).round() as u8;

                                *pixel =
                                    tiny_skia::PremultipliedColorU8::from_rgba(fr8, fg8, fb8, fa8)
                                        .unwrap_or(tiny_skia::PremultipliedColorU8::TRANSPARENT);
                            } else {
                                *pixel = tiny_skia::PremultipliedColorU8::TRANSPARENT;
                            }
                        }

                        if shadow.blur_radius > 0.0 {
                            crate::fast_box_blur::fast_box_blur(
                                new_shadow_pixmap.pixels_mut(),
                                sw as usize,
                                sh as usize,
                                shadow.blur_radius.round() as u32,
                            );
                        }

                        // Write back to cache
                        {
                            let mut cache = handle.cache.write();
                            if let Some(mut clone_to_cache) = tiny_skia::Pixmap::new(sw, sh) {
                                clone_to_cache
                                    .data_mut()
                                    .copy_from_slice(new_shadow_pixmap.data());
                                cache.shadow_pixmap =
                                    Some((clone_to_cache, (scale_x, scale_y), shadow.blur_radius));
                            }
                        }

                        new_shadow_pixmap
                    };

                    // Draw shadow
                    let offset_x = x - pad + shadow.offset_x;
                    let offset_y = y - pad + shadow.offset_y;

                    let pattern = tiny_skia::Pattern::new(
                        shadow_pixmap_to_draw.as_ref(), // Use the cached or newly rendered shadow buffer
                        tiny_skia::SpreadMode::Pad,
                        tiny_skia::FilterQuality::Bilinear,
                        1.0,
                        tiny_skia::Transform::from_translate(offset_x, offset_y),
                    );

                    let mut paint = tiny_skia::Paint::default();
                    paint.shader = pattern;
                    paint.anti_alias = true;

                    let mut pb = tiny_skia::PathBuilder::new();
                    if let Some(rect) =
                        tiny_skia::Rect::from_xywh(offset_x, offset_y, sw as f32, sh as f32)
                    {
                        pb.push_rect(rect);
                        if let Some(path) = pb.finish() {
                            self.with_current_pixmap(|p, clip| {
                                p.fill_path(
                                    &path,
                                    &paint,
                                    tiny_skia::FillRule::Winding,
                                    transform,
                                    clip,
                                );
                            });
                        }
                    }
                }
            }
        }

        // 2. Draw actual SVG

        // Scope for SVG body cache
        let mut cached_body = None;
        let tw = target_width.ceil() as u32;
        let th = target_height.ceil() as u32;

        if tw > 0 && th > 0 {
            {
                let cache = handle.cache.read();
                if let Some((ref pixmap, (sx, sy))) = cache.svg_pixmap {
                    if (sx - scale_x).abs() < 0.001 && (sy - scale_y).abs() < 0.001 {
                        if let Some(mut clone_pixmap) =
                            tiny_skia::Pixmap::new(pixmap.width(), pixmap.height())
                        {
                            clone_pixmap.data_mut().copy_from_slice(pixmap.data());
                            cached_body = Some(clone_pixmap);
                        }
                    }
                }
            }

            let body_pixmap_to_draw = if let Some(cached) = cached_body {
                cached
            } else {
                let mut new_body_pixmap = tiny_skia::Pixmap::new(tw, th).unwrap();
                new_body_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                let svg_render_transform = tiny_skia::Transform::from_scale(scale_x, scale_y);
                resvg::render(
                    &handle.tree,
                    svg_render_transform,
                    &mut new_body_pixmap.as_mut(),
                );

                {
                    let mut cache = handle.cache.write();
                    if let Some(mut clone_to_cache) = tiny_skia::Pixmap::new(tw, th) {
                        clone_to_cache
                            .data_mut()
                            .copy_from_slice(new_body_pixmap.data());
                        cache.svg_pixmap = Some((clone_to_cache, (scale_x, scale_y)));
                    }
                }

                new_body_pixmap
            };

            // Draw original SVG from cache
            let final_transform = transform.pre_translate(x, y);

            let pattern = tiny_skia::Pattern::new(
                body_pixmap_to_draw.as_ref(),
                tiny_skia::SpreadMode::Pad,
                tiny_skia::FilterQuality::Bilinear,
                1.0,
                final_transform,
            );

            let mut paint = tiny_skia::Paint::default();
            paint.shader = pattern;
            paint.anti_alias = true;

            let mut pb = tiny_skia::PathBuilder::new();
            if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, tw as f32, th as f32) {
                pb.push_rect(rect);
                if let Some(path) = pb.finish() {
                    self.with_current_pixmap(|p, clip| {
                        p.fill_path(&path, &paint, tiny_skia::FillRule::Winding, transform, clip);
                    });
                }
            }
        }

        Ok(())
    }

    fn draw_text(
        &mut self,
        text: &str,
        text_format: &mut Self::TextFormatHandle,
        left: f32,
        top: f32,
        width: f32,
        height: f32,
        brush: &Self::BrushHandle,
        _options: Option<&TextDrawOptions>,
    ) -> Result<(), Self::Error> {
        let (dpi_x, dpi_y) = self.dpi_scale;
        let font_size = text_format.font_size * dpi_y;
        let phys_width = width * dpi_x;
        let phys_height = height * dpi_y;

        let mut buffer = cosmic_text::Buffer::new(
            &mut self.font_system,
            cosmic_text::Metrics::new(font_size, font_size * text_format.line_height_factor),
        );
        buffer.set_size(
            &mut self.font_system,
            if width > 0.0 { Some(phys_width) } else { None },
            if height > 0.0 {
                Some(phys_height)
            } else {
                None
            },
        );
        buffer.set_wrap(&mut self.font_system, text_format.to_cosmic_wrap());
        buffer.set_text(
            &mut self.font_system,
            text,
            &text_format.to_cosmic_attrs(),
            cosmic_text::Shaping::Advanced,
            text_format.to_cosmic_align(),
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut total_h = 0.0f32;
        for run in buffer.layout_runs() {
            total_h += run.line_height;
        }

        let offset_y = match text_format.paragraph_alignment {
            ParagraphAlignment::Center if phys_height > total_h => (phys_height - total_h) / 2.0,
            ParagraphAlignment::Bottom if phys_height > total_h => phys_height - total_h,
            _ => 0.0,
        };

        let mut paint = tiny_skia::Paint::default();
        paint.anti_alias = true;

        let glyph_color = match brush {
            TinySkiaBrushHandle::Solid(color) => {
                paint.set_color(*color);
                Some(*color)
            }
            TinySkiaBrushHandle::Gradient { .. } => None,
        };

        let transform = self.get_current_transform();

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph =
                    glyph.physical((left * dpi_x, top * dpi_y + offset_y + run.line_y), 1.0);

                if let Some(image) = self
                    .swash_cache
                    .get_image(&mut self.font_system, physical_glyph.cache_key)
                {
                    let gx = physical_glyph.x as f32 + image.placement.left as f32;
                    let gy = physical_glyph.y as f32 - image.placement.top as f32;

                    let logical_gx = gx / dpi_x;
                    let logical_gy = gy / dpi_y;

                    if image.placement.width == 0 || image.placement.height == 0 {
                        continue;
                    }

                    if let Some(mut pixmap) =
                        tiny_skia::Pixmap::new(image.placement.width, image.placement.height)
                    {
                        let pixels = pixmap.pixels_mut();

                        let mut i = 0;
                        match image.content {
                            cosmic_text::SwashContent::SubpixelMask => {
                                for _y in 0..image.placement.height {
                                    for _x in 0..image.placement.width {
                                        let r = image.data[i * 3];
                                        let g = image.data[i * 3 + 1];
                                        let b = image.data[i * 3 + 2];
                                        let a =
                                            ((r as u32 + g as u32 + b as u32) / 3) as f32 / 255.0;
                                        let fa = (a * 255.0).round() as u8;
                                        if let Some(c) = glyph_color {
                                            let mut p_color = c;
                                            p_color.apply_opacity(a);
                                            pixels[i] = p_color.premultiply().to_color_u8();
                                        } else {
                                            // Mask for gradient
                                            pixels[i] = tiny_skia::PremultipliedColorU8::from_rgba(
                                                fa, fa, fa, fa,
                                            )
                                            .unwrap_or(
                                                tiny_skia::PremultipliedColorU8::TRANSPARENT,
                                            );
                                        }
                                        i += 1;
                                    }
                                }
                            }
                            cosmic_text::SwashContent::Mask => {
                                for _y in 0..image.placement.height {
                                    for _x in 0..image.placement.width {
                                        let a = image.data[i] as f32 / 255.0;
                                        let fa = image.data[i];
                                        if let Some(c) = glyph_color {
                                            let mut p_color = c;
                                            p_color.apply_opacity(a);
                                            pixels[i] = p_color.premultiply().to_color_u8();
                                        } else {
                                            // Mask for gradient
                                            pixels[i] = tiny_skia::PremultipliedColorU8::from_rgba(
                                                fa, fa, fa, fa,
                                            )
                                            .unwrap_or(
                                                tiny_skia::PremultipliedColorU8::TRANSPARENT,
                                            );
                                        }
                                        i += 1;
                                    }
                                }
                            }
                            cosmic_text::SwashContent::Color => {
                                for _y in 0..image.placement.height {
                                    for _x in 0..image.placement.width {
                                        let r = image.data[i * 4];
                                        let g = image.data[i * 4 + 1];
                                        let b = image.data[i * 4 + 2];
                                        let a = image.data[i * 4 + 3];
                                        let color = tiny_skia::Color::from_rgba8(r, g, b, a);
                                        pixels[i] = color.premultiply().to_color_u8();
                                        i += 1;
                                    }
                                }
                            }
                        }

                        // Apply gradient shader if we are not solid color
                        if glyph_color.is_none() {
                            let mut stops = Vec::new();
                            let mut local_shader = None;

                            // To map the local glyph coordinates (0, 0) to world coordinates for the shader evaluation,
                            // The shader transform takes generic points to world points.
                            // The point `(0, 0)` in our pixmap will be placed at `transform.pre_translate(gx, gy)`
                            // So the transform for the shader is precisely `transform.pre_translate(gx, gy)`
                            // inverted? No, the transform passed to `LinearGradient::new` maps the gradient's local shape to the destination geometry!
                            //
                            // Wait! In TinySkia, the gradient's start/end are directly specified in WORLD/Destination coordinates if transform is identity.
                            // However, we are drawing into a temporary pixmap where (0,0) represents the WORLD coordinate `transform.map_point(gx, gy)`.
                            // So, the world coordinate `(X_world, Y_world)` corresponds to local `(X_world - px, Y_world - py)` mapped back.
                            // Let's formulate the transform: if a point `p_local` in the pixmap maps to `p_world = transform * translate(gx,gy) * p_local`,
                            // then the inverse mapping from `p_local` back to the Shader's space must mimic the global context.
                            // Because tiny_skia `fill_rect` uses an Identity transform for the Rect relative to the Shader,
                            // the shader evaluates at `p_local`.
                            // So we want the shader at `p_local` to evaluate as if it was at `p_world`.
                            // That means `p_local -> p_world` should be the Transform supplied to the Gradient!

                            let shader_transform = transform
                                .pre_translate(logical_gx, logical_gy)
                                .pre_scale(1.0 / dpi_x, 1.0 / dpi_y)
                                .invert()
                                .unwrap_or(tiny_skia::Transform::identity());

                            if let TinySkiaBrushHandle::Gradient { gradient } = brush {
                                match gradient {
                                    flor_base::graphics::Gradient::Linear {
                                        start,
                                        end,
                                        colors,
                                    } => {
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
                                    flor_base::graphics::Gradient::Radial {
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
                                            *radius,
                                            tiny_skia::Point::from_xy(center.0, center.1),
                                            0.0,
                                            stops,
                                            tiny_skia::SpreadMode::Pad,
                                            shader_transform,
                                        );
                                    }
                                }
                            }

                            if let Some(shader) = local_shader {
                                let mut mask_paint = tiny_skia::Paint::default();
                                mask_paint.shader = shader;
                                mask_paint.blend_mode = tiny_skia::BlendMode::SourceIn;
                                mask_paint.anti_alias = true;

                                if let Some(rect) = tiny_skia::Rect::from_xywh(
                                    0.0,
                                    0.0,
                                    image.placement.width as f32,
                                    image.placement.height as f32,
                                ) {
                                    // Use Transform::identity() because rect is purely local to the pixmap, and the shader transform handles mapping.
                                    pixmap.fill_rect(
                                        rect,
                                        &mask_paint,
                                        tiny_skia::Transform::identity(),
                                        None,
                                    );
                                }
                            }
                        }

                        // draw the glyph pixmap onto the target surface
                        let mut local_transform = transform;
                        // Transform sets logical position, then decodes the DPI scale for the physical pixmap
                        local_transform = local_transform
                            .pre_translate(logical_gx, logical_gy)
                            .pre_scale(1.0 / dpi_x, 1.0 / dpi_y);

                        self.with_current_pixmap(|p, clip| {
                            p.draw_pixmap(
                                0,
                                0,
                                pixmap.as_ref(),
                                &tiny_skia::PixmapPaint::default(),
                                local_transform,
                                clip,
                            );
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn draw_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        stroke_width: f32,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        let ts_path = if let Some(p) = Self::build_tiny_skia_path(path) {
            p
        } else {
            return Ok(());
        };

        let mut paint = tiny_skia::Paint::default();
        paint.anti_alias = true;
        match brush {
            TinySkiaBrushHandle::Solid(color) => {
                paint.set_color(*color);
            }
            TinySkiaBrushHandle::Gradient { gradient } => {
                let mut stops = Vec::new();
                match gradient {
                    flor_base::graphics::Gradient::Linear { start, end, colors } => {
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
                    flor_base::graphics::Gradient::Radial {
                        center,
                        radius,
                        colors,
                    } => {
                        for (pos, c) in colors {
                            stops.push(tiny_skia::GradientStop::new(*pos, c.to_tiny_skia()));
                        }
                        if let Some(shader) = tiny_skia::RadialGradient::new(
                            tiny_skia::Point::from_xy(center.0, center.1),
                            *radius,
                            tiny_skia::Point::from_xy(center.0, center.1),
                            0.0,
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

        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = stroke_width;

        let transform = if let Some(opts) = options {
            if let Some(t) = &opts.transform {
                self.get_current_transform().pre_concat(t.to_tiny_skia())
            } else {
                self.get_current_transform()
            }
        } else {
            self.get_current_transform()
        };

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
        let ts_path = if let Some(p) = Self::build_tiny_skia_path(path) {
            p
        } else {
            return Ok(());
        };

        let mut paint = tiny_skia::Paint::default();
        paint.anti_alias = true;

        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let _ = self.draw_shadow(path, shadow, opts.transform.as_ref());
            }
        }

        match brush {
            TinySkiaBrushHandle::Solid(color) => {
                paint.set_color(*color);
            }
            TinySkiaBrushHandle::Gradient { gradient } => {
                let mut stops = Vec::new();
                match gradient {
                    flor_base::graphics::Gradient::Linear { start, end, colors } => {
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
                    flor_base::graphics::Gradient::Radial {
                        center,
                        radius,
                        colors,
                    } => {
                        for (pos, c) in colors {
                            stops.push(tiny_skia::GradientStop::new(*pos, c.to_tiny_skia()));
                        }
                        if let Some(shader) = tiny_skia::RadialGradient::new(
                            tiny_skia::Point::from_xy(center.0, center.1),
                            *radius,
                            tiny_skia::Point::from_xy(center.0, center.1),
                            0.0,
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

        let transform = if let Some(opts) = options {
            if let Some(t) = &opts.transform {
                self.get_current_transform().pre_concat(t.to_tiny_skia())
            } else {
                self.get_current_transform()
            }
        } else {
            self.get_current_transform()
        };

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
        let mut stroke = tiny_skia::Stroke::default();
        stroke.width = border_width;

        let transform = if let Some(opts) = options {
            if let Some(t) = &opts.transform {
                self.get_current_transform().pre_concat(t.to_tiny_skia())
            } else {
                self.get_current_transform()
            }
        } else {
            self.get_current_transform()
        };

        let mut paint = tiny_skia::Paint::default();
        paint.anti_alias = true;

        match brush {
            TinySkiaBrushHandle::Solid(color) => {
                paint.set_color(*color);
            }
            TinySkiaBrushHandle::Gradient { gradient } => {
                let mut stops = Vec::new();
                match gradient {
                    flor_base::graphics::Gradient::Linear { start, end, colors } => {
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
                    flor_base::graphics::Gradient::Radial {
                        center,
                        radius,
                        colors,
                    } => {
                        for (pos, c) in colors {
                            stops.push(tiny_skia::GradientStop::new(*pos, c.to_tiny_skia()));
                        }
                        if let Some(shader) = tiny_skia::RadialGradient::new(
                            tiny_skia::Point::from_xy(center.0, center.1),
                            *radius,
                            tiny_skia::Point::from_xy(center.0, center.1),
                            0.0,
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
        let transform = if let Some(opts) = options {
            if let Some(t) = &opts.transform {
                self.get_current_transform().pre_concat(t.to_tiny_skia())
            } else {
                self.get_current_transform()
            }
        } else {
            self.get_current_transform()
        };

        let mut paint = tiny_skia::Paint::default();
        paint.anti_alias = true;
        match brush {
            TinySkiaBrushHandle::Solid(color) => {
                paint.set_color(*color);
            }
            TinySkiaBrushHandle::Gradient { gradient } => {
                let mut stops = Vec::new();
                match gradient {
                    flor_base::graphics::Gradient::Linear { start, end, colors } => {
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
                    flor_base::graphics::Gradient::Radial {
                        center,
                        radius,
                        colors,
                    } => {
                        for (pos, c) in colors {
                            stops.push(tiny_skia::GradientStop::new(*pos, c.to_tiny_skia()));
                        }
                        if let Some(shader) = tiny_skia::RadialGradient::new(
                            tiny_skia::Point::from_xy(center.0, center.1),
                            *radius,
                            tiny_skia::Point::from_xy(center.0, center.1),
                            0.0,
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

        let r = corner_radius.unwrap_or(0.0);

        if let Some(opts) = options {
            if let Some(shadow) = &opts.shadow {
                let path = Path::from_rounded_rect(left, top, width, height, r);
                let _ = self.draw_shadow(&path, shadow, opts.transform.as_ref());
            }
        }

        if r <= 0.0 {
            // Fast path for flat rectangle
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
        let path = Path::from_rounded_rect(left, top, width, height, corner_radius);
        self.blur_path(&path, blur_radius, transform)
    }

    fn blur_path(
        &mut self,
        path: &Path,
        blur_radius: f32,
        options_transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error> {
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
        let pad = blur_radius.ceil() as f32 * 2.5;

        let left = (bounds.left() - pad).floor() as i32;
        let top = (bounds.top() - pad).floor() as i32;
        let right = (bounds.right() + pad).ceil() as i32;
        let bottom = (bounds.bottom() + pad).ceil() as i32;

        let mut extracted_pixels: Option<tiny_skia::Pixmap> = None;
        let mut actual_left = 0.0;
        let mut actual_top = 0.0;

        let w = (right - left) as u32;
        let h = (bottom - top) as u32;

        if w > 0 && h > 0 {
            if let Some(mut pixmap) = tiny_skia::Pixmap::new(w, h) {
                // Initialize to transparent black
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
            crate::fast_box_blur::fast_box_blur(
                blur_pixmap.pixels_mut(),
                bw,
                bh,
                blur_radius.round() as u32,
            );

            let mut pattern_paint = tiny_skia::Paint::default();
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
        let path = Path::from_rounded_rect(rect.0, rect.1, rect.2, rect.3, radius);
        if let Some(ts_path) = Self::build_tiny_skia_path(&path) {
            self.push_path_internal(ts_path);
        }
        Ok(())
    }

    fn push_path_clip(&mut self, path: &Path) -> Result<(), Self::Error> {
        if let Some(ts_path) = Self::build_tiny_skia_path(path) {
            self.push_path_internal(ts_path);
        }
        Ok(())
    }

    fn pop_clip(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
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
        Ok(self.clip_stack.len() as u32)
    }

    fn suspend_clip(&mut self) -> Result<(), Self::Error> {
        self.clip_suspended = true;
        self.update_clip_mask();
        Ok(())
    }

    fn resume_clip(&mut self) -> Result<(), Self::Error> {
        self.clip_suspended = false;
        self.update_clip_mask();
        Ok(())
    }

    fn push_transform(&mut self, transform: &Transform2D) -> Result<(), Self::Error> {
        let ts = transform.to_tiny_skia();
        let current = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(tiny_skia::Transform::identity());
        // Depending on pre or post multiplication order, usually it's current.pre_concat(ts)
        self.transform_stack.push(current.pre_concat(ts));
        Ok(())
    }

    fn pop_transform(&mut self, _target_depth: Option<u32>) -> Result<(), Self::Error> {
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
        Ok(self.transform_stack.len() as u32)
    }

    fn capture_snapshot(
        &mut self,
        _rect: Option<(f32, f32, u32, u32)>,
    ) -> Result<Vec<u8>, Self::Error> {
        // currently ignoring optional rect and snapshotting whole default_pixmap
        // to return RGBA bytes
        let data = self.default_pixmap.data();
        let mut out = Vec::with_capacity(data.len());
        // Un-premultiply
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
