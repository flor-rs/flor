use crate::error::GlError;
use crate::handle::GlImageHandle;
use crate::handle::GlSurfaceId;
use crate::handle::{GlBrushHandle, GlTextFormatHandle};
use cosmic_text::{CacheKey, FontSystem, Placement, SwashCache, SwashContent};
use flor_base::graphics::{
    Gradient, HitTestResult, ImageDrawOptions, Path, PathDrawOptions, Render, RenderContext,
    ScaleMode, TextDrawOptions,
};
use flor_base::types::{Color, Transform2D};
use glow::{HasContext, NativeTexture, PixelUnpackData};
use parking_lot::Mutex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::slice;
use std::sync::{Arc, OnceLock};
use windows::Win32::Foundation::HWND;

use crate::display_context::{DisplayContext, NativeDisplayContext};
use crate::shader::builtin::{
    BLUR_FRAGMENT_SHADER, COLOR_FRAGMENT_SHADER, COLOR_VERTEX_SHADER, TEXTURE_FRAGMENT_SHADER,
    TEXTURE_VERTEX_SHADER, TEXT_FRAGMENT_SHADER,
};
use crate::shader::tessellator::{Tessellator, Vertex};
use crate::shader::ShaderProgram;

#[cfg(feature = "svg")]
use {crate::handle::GlSvgHandle, flor_base::graphics::SvgDrawOptions};

mod config;
use crate::renderer::context::GlContext;
pub use config::*;

mod context;

pub(crate) static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

macro_rules! time_it {
    ($name:literal) => {
        let _timer = crate::renderer::utils::Timer::new($name);
    };
}

// 无缩放时的dpi倍率
const NO_SCALE_DPI: (f32, f32) = (1.0, 1.0);

// #[derive(Debug)] removed for manual implementation
pub struct GlRenderer {
    display_context: NativeDisplayContext,
    pub gl_context: GlContext,

    // Scale and Viewport
    // dpi 缩放倍率，这里是倍率
    pub dpi_scale: (f32, f32),
    pub window_size: (u32, u32),

    pub proj_transform: Transform2D,
    // Transform Stack
    pub transform_stack: Vec<Transform2D>,
    // 负责dpi全局缩放的矩阵，GlRenderer.dpi_scale == NO_SCALE_DPI 时，应该为None
    pub scale_transform: Option<Transform2D>,

    // Text rendering caches
    pub swash_cache: SwashCache,
    pub glyph_cache: HashMap<CacheKey, (glow::Texture, Placement)>,

    // Pipelines
    pub text_program: ShaderProgram,
    pub texture_program: ShaderProgram,
    pub color_program: ShaderProgram,
    pub blur_program: ShaderProgram,
    pub text_vao: glow::VertexArray,
    pub text_vbo: glow::Buffer,
    pub color_vao: glow::VertexArray,
    pub color_vbo: glow::Buffer,
    pub tessellator: Tessellator,

    // Rendering State
    pub wait_v_sync: bool,

    pub image_texture_cache: HashMap<(u64, usize), glow::Texture>,
    pub next_image_id: u64,
}

impl std::fmt::Debug for GlRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlRenderer")
            .field("dpi_scale", &self.dpi_scale)
            .field("transform_stack", &self.transform_stack)
            .finish()
    }
}

impl Render for GlRenderer {
    #[cfg(target_os = "windows")]
    type HWND = HWND;
    type Render = GlRenderer;
    type Config = GlConfig;

    fn create(
        hwnd: impl Into<Self::HWND>,
        width: u32,
        height: u32,
        wait_v_sync: bool,
        config: Self::Config,
    ) -> Result<Self::Render, Self::Error> {
        let display_context = NativeDisplayContext::create(hwnd.into(), config)?;
        let gl_context = GlContext::from_context(display_context.get_gl_context());
        unsafe {
            display_context.set_v_sync(wait_v_sync);
            gl_context.viewport(0, 0, width as i32, height as i32);

            gl_context.enable(glow::BLEND);
            gl_context.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        let text_program =
            ShaderProgram::new(&gl_context, TEXTURE_VERTEX_SHADER, TEXT_FRAGMENT_SHADER)?;

        let texture_program =
            ShaderProgram::new(&gl_context, TEXTURE_VERTEX_SHADER, TEXTURE_FRAGMENT_SHADER)?;

        let text_vao = unsafe { gl_context.create_vertex_array()? };
        let text_vbo = unsafe { gl_context.create_buffer()? };

        let color_program =
            ShaderProgram::new(&gl_context, COLOR_VERTEX_SHADER, COLOR_FRAGMENT_SHADER)?;

        let blur_program =
            ShaderProgram::new(&gl_context, COLOR_VERTEX_SHADER, BLUR_FRAGMENT_SHADER)?;

        let color_vao = unsafe { gl_context.create_vertex_array()? };
        let color_vbo = unsafe { gl_context.create_buffer()? };

        unsafe {
            gl_context.bind_vertex_array(Some(text_vao));
            gl_context.bind_buffer(glow::ARRAY_BUFFER, Some(text_vbo));
            let stride = 4 * size_of::<f32>() as i32;
            gl_context.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl_context.enable_vertex_attrib_array(0);
            gl_context.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                stride,
                2 * size_of::<f32>() as i32,
            );
            gl_context.enable_vertex_attrib_array(1);
            gl_context.bind_vertex_array(None);

            gl_context.bind_vertex_array(Some(color_vao));
            gl_context.bind_buffer(glow::ARRAY_BUFFER, Some(color_vbo));
            let stride = size_of::<Vertex>() as i32;
            gl_context.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl_context.enable_vertex_attrib_array(0);
            gl_context.vertex_attrib_pointer_f32(
                1,
                4,
                glow::FLOAT,
                false,
                stride,
                2 * size_of::<f32>() as i32,
            );
            gl_context.enable_vertex_attrib_array(1);
            gl_context.bind_vertex_array(None);
        }

        FONT_SYSTEM.get_or_init(|| Mutex::new(FontSystem::new()));

        let proj_transform = Transform2D::ortho(width as f32, height as f32);

        let mut renderer = GlRenderer {
            display_context,
            gl_context,
            dpi_scale: (1.0, 1.0),
            window_size: (width, height),
            proj_transform,
            transform_stack: vec![],
            scale_transform: None,
            swash_cache: SwashCache::new(),
            glyph_cache: HashMap::new(),
            text_program,
            texture_program,
            color_program,
            blur_program,
            text_vao,
            text_vbo,
            color_vao,
            color_vbo,
            tessellator: Tessellator::new(),
            wait_v_sync,

            image_texture_cache: HashMap::new(),
            next_image_id: 1,
        };

        // 立即根据初始宽高构建 FBO
        let _ = renderer.update_window_size(width, height);

        Ok(renderer)
    }
}

impl RenderContext for GlRenderer {
    type Error = GlError;
    type ImageHandle = GlImageHandle;
    type SurfaceId = GlSurfaceId;
    type BrushHandle = GlBrushHandle;
    #[cfg(feature = "svg")]
    type SvgHandle = GlSvgHandle;
    type TextFormatHandle = GlTextFormatHandle;

    fn begin(&mut self) -> Result<(), Self::Error> {
        time_it!("begin");
        Ok(())
    }

    fn end(&mut self) -> Result<(), Self::Error> {
        time_it!("end");
        self.display_context.present();
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        time_it!("clear");
        unsafe {
            self.gl_context.clear_color(
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            );
            self.gl_context.clear(glow::COLOR_BUFFER_BIT);
        }
        Ok(())
    }

    fn test(&mut self) -> Result<(), Self::Error> {
        // Reserved for debugging
        Ok(())
    }

    fn update_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        time_it!("update_window_size");
        self.window_size = (width, height);
        unsafe {
            let proj_transform = Transform2D::ortho(width as f32, height as f32);
            self.proj_transform = proj_transform;
            self.gl_context.viewport(0, 0, width as i32, height as i32);
        }
        Ok(())
    }

    fn set_scale_factor(&mut self, dpi_x: f32, dpi_y: f32) -> Result<(), Self::Error> {
        self.dpi_scale = (dpi_x, dpi_y);

        Ok(())
    }

    fn create_surface(&mut self, width: u32, height: u32) -> Result<Self::SurfaceId, Self::Error> {
        Ok(GlSurfaceId {})
    }

    fn set_render_target(&mut self, surface_id: &Self::SurfaceId) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset_render_target(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn create_image_from_bytes(&mut self, bytes: &[u8]) -> Result<Self::ImageHandle, Self::Error> {
        time_it!("create_image_from_bytes");
        let id = self.next_image_id;
        self.next_image_id += 1;

        Ok(GlImageHandle::new(id, bytes)?)
    }

    fn create_image_from_raw_bytes(
        &mut self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<Self::ImageHandle, Self::Error> {
        let id = self.next_image_id;
        self.next_image_id += 1;
        let total_delays = delays.iter().map(|&d| d as u128).sum();

        Ok(GlImageHandle {
            id,
            width,
            height,
            delays,
            total_delays,
            frames: Arc::new(raw_bytes.clone()),
            cache: Mutex::new(Vec::new()),
        })
    }

    #[cfg(feature = "svg")]
    fn create_svg(&mut self, bytes: &[u8]) -> Result<Self::SvgHandle, Self::Error> {
        time_it!("create_svg");
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(bytes, &opt)
            .map_err(|e| GlError::CustomError(format!("SVG parse error: {:?}", e)))?;

        Ok(GlSvgHandle {
            cache: Mutex::new(None),
            tree: Arc::new(tree),
        })
    }

    fn create_text_format(
        &mut self,
        font_family_name: &str,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        let mut handle = GlTextFormatHandle::new();
        handle.font_family_name = font_family_name.to_string();
        Ok(handle)
    }

    #[cfg(feature = "memory-font")]
    fn create_text_format_from_bytes(
        &mut self,
        font_data: &[u8],
        _ttc_index: u32,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        let source = cosmic_text::fontdb::Source::Binary(Arc::new(font_data.to_vec()));
        let mut handle = GlTextFormatHandle::new();
        if let Some(mut font_system_lock) = FONT_SYSTEM.get().map(|f| f.lock()) {
            let ids = font_system_lock.db_mut().load_font_source(source);
            if !ids.is_empty() {
                handle.custom_font_id = Some(ids[0]);
            }
        }
        Ok(handle)
    }

    fn measure_text(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
    ) -> Result<(f32, f32), Self::Error> {
        time_it!("measure_text");
        let (dpi_x, dpi_y) = self.dpi_scale;
        let Some(mut font_system_lock) = FONT_SYSTEM.get().map(|f| f.lock()) else {
            return Ok((0.0, 0.0));
        };
        let buffer = Self::prepare_text_buffer(
            &mut font_system_lock,
            text,
            text_format,
            width,
            height,
            dpi_x,
            dpi_y,
        );

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
        time_it!("hit_test_point");
        let (dpi_x, dpi_y) = self.dpi_scale;
        let phys_height = height * dpi_y;

        let Some(mut font_system_lock) = FONT_SYSTEM.get().map(|f| f.lock()) else {
            return Ok(HitTestResult {
                is_inside: false,
                text_index: 0,
                is_trailing: false,
                is_trimmed: false,
                rect: (0.0, 0.0, 0.0, 0.0),
            });
        };
        let buffer = Self::prepare_text_buffer(
            &mut font_system_lock,
            text,
            text_format,
            width,
            height,
            dpi_x,
            dpi_y,
        );

        let mut total_h = 0.0f32;
        for run in buffer.layout_runs() {
            total_h += run.line_height;
        }

        let offset_y = match text_format.paragraph_alignment {
            flor_base::graphics::ParagraphAlignment::Center if phys_height > total_h => {
                (phys_height - total_h) / 2.0
            }
            flor_base::graphics::ParagraphAlignment::Bottom if phys_height > total_h => {
                phys_height - total_h
            }
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
        time_it!("hit_test_text_position");
        let (dpi_x, dpi_y) = self.dpi_scale;
        let phys_height = height * dpi_y;

        let Some(mut font_system_lock) = FONT_SYSTEM.get().map(|f| f.lock()) else {
            return Ok((0.0, 0.0));
        };
        let buffer = Self::prepare_text_buffer(
            &mut font_system_lock,
            text,
            text_format,
            width,
            height,
            dpi_x,
            dpi_y,
        );

        let mut total_h = 0.0f32;
        for run in buffer.layout_runs() {
            total_h += run.line_height;
        }

        let offset_y = match text_format.paragraph_alignment {
            flor_base::graphics::ParagraphAlignment::Center if phys_height > total_h => {
                (phys_height - total_h) / 2.0
            }
            flor_base::graphics::ParagraphAlignment::Bottom if phys_height > total_h => {
                phys_height - total_h
            }
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
        _opacity: Option<f32>,
    ) -> Result<Self::BrushHandle, Self::Error> {
        Ok(GlBrushHandle::Solid(color))
    }

    fn create_gradient_brush(
        &mut self,
        gradient: &Gradient,
    ) -> Result<Self::BrushHandle, Self::Error> {
        Ok(GlBrushHandle::Gradient {
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
        time_it!("draw_image");
        let target_w = width.unwrap_or(handle.width as f32);
        let target_h = height.unwrap_or(handle.height as f32);

        if target_w <= 0.0 || target_h <= 0.0 || handle.width == 0 || handle.height == 0 {
            return Ok(());
        }

        let scale_mode = options
            .and_then(|o| o.scale_mode)
            .unwrap_or(ScaleMode::Stretch);

        let img_w = handle.width as f32;
        let img_h = handle.height as f32;

        let (draw_x, draw_y, draw_w, draw_h) =
            scale_mode.calc_draw_rect(x, y, target_w, target_h, img_w, img_h);

        let frame_index = options
            .and_then(|o| o.frame_index)
            .unwrap_or(0)
            .min(handle.frames.len().saturating_sub(1));

        let texture = {
            let mut cache = handle.cache.lock();
            if cache.is_empty() {
                // 初次渲染时，一次性把图片的所有帧加载到显存里！
                for frame_data in handle.frames.iter() {
                    let tex = self.gl_context.create_texture_tex_image_2d(
                        img_w as i32,
                        img_h as i32,
                        PixelUnpackData::Slice(Some(frame_data)),
                    )?;
                    cache.push(tex);
                }
            }
            cache[frame_index]
        };

        let opacity = options.and_then(|o| o.opacity);
        self.draw_texture(draw_x, draw_y, draw_w, draw_h, texture, opacity);
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
        time_it!("draw_svg");
        let scale_mode = options
            .and_then(|o| o.scale_mode)
            .unwrap_or(ScaleMode::Stretch);

        // 分辨率无关适配，实际的像素尺寸映射
        let img_w = handle.tree.size().width();
        let img_h = handle.tree.size().height();

        let target_w = width.unwrap_or(img_w);
        let target_h = height.unwrap_or(img_h);

        if target_w <= 0.0 || target_h <= 0.0 || img_w <= 0.0 || img_h <= 0.0 {
            return Ok(());
        }

        let (draw_x, draw_y, draw_w, draw_h) =
            scale_mode.calc_draw_rect(x, y, target_w, target_h, img_w, img_h);

        let physical_w = draw_w.ceil() as u32;
        let physical_h = draw_h.ceil() as u32;

        let texture = {
            let mut option_texture = handle.cache.lock();

            match option_texture.as_ref().copied() {
                Some((w, h, tex)) if w == physical_w && h == physical_h => tex,
                _ => {
                    let mut pixmap =
                        resvg::tiny_skia::Pixmap::new(physical_w.max(1), physical_h.max(1))
                            .unwrap();
                    let sx = physical_w as f32 / img_w;
                    let sy = physical_h as f32 / img_h;

                    let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
                    resvg::render(&handle.tree, transform, &mut pixmap.as_mut());

                    let new_texture = unsafe {
                        self.gl_context.create_texture_tex_image_2d(
                            physical_w as i32,
                            physical_h as i32,
                            PixelUnpackData::Slice(Some(pixmap.data())),
                        )?
                    };

                    if let Some((_, _, old_tex)) =
                        option_texture.replace((physical_w, physical_h, new_texture))
                    {
                        unsafe {
                            self.gl_context.delete_texture(old_tex);
                        }
                    }
                    new_texture
                }
            }
        };

        let opacity = options.and_then(|o| o.opacity);
        self.draw_texture(draw_x, draw_y, draw_w, draw_h, texture, opacity);
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
        time_it!("draw_text");
        let (dpi_x, dpi_y) = self.dpi_scale;
        let phys_height = height * dpi_y;

        let Some(mut font_system_lock) = FONT_SYSTEM.get().map(|f| f.lock()) else {
            return Ok(());
        };
        let buffer = Self::prepare_text_buffer(
            &mut font_system_lock,
            text,
            text_format,
            width,
            height,
            dpi_x,
            dpi_y,
        );

        let offset_y = text_format.calc_offset_y(&buffer, phys_height);

        let brush_data = brush.to_shader_data();

        let mut glyphs_to_draw = Vec::new();

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph =
                    glyph.physical((left * dpi_x, top * dpi_y + offset_y + run.line_y), 1.0);

                let (texture, placement) = match self.glyph_cache.entry(physical_glyph.cache_key) {
                    Entry::Occupied(o) => o.get().clone(),
                    Entry::Vacant(v) => {
                        if let Some(image) = self
                            .swash_cache
                            .get_image(&mut font_system_lock, physical_glyph.cache_key)
                        {
                            let w = image.placement.width;
                            let h = image.placement.height;
                            if w == 0 || h == 0 {
                                continue;
                            }

                            let mut rgba_data = Vec::with_capacity((w * h * 4) as usize);
                            match image.content {
                                SwashContent::Mask => {
                                    for &a in &image.data {
                                        rgba_data.push(255);
                                        rgba_data.push(255);
                                        rgba_data.push(255);
                                        rgba_data.push(a);
                                    }
                                }
                                SwashContent::SubpixelMask => {
                                    for chunk in image.data.chunks_exact(3) {
                                        let a =
                                            ((chunk[0] as u32 + chunk[1] as u32 + chunk[2] as u32)
                                                / 3)
                                                as u8;
                                        rgba_data.push(255);
                                        rgba_data.push(255);
                                        rgba_data.push(255);
                                        rgba_data.push(a);
                                    }
                                }
                                SwashContent::Color => {
                                    rgba_data.extend_from_slice(&image.data);
                                }
                            }

                            let texture = self.gl_context.create_texture_tex_image_2d(
                                w as i32,
                                h as i32,
                                PixelUnpackData::Slice(Some(&rgba_data[..])),
                            )?;
                            v.insert((texture, image.placement));
                            (texture, image.placement)
                        } else {
                            continue;
                        }
                    }
                };

                let w = placement.width as f32;
                let h = placement.height as f32;

                let gx = physical_glyph.x as f32 + placement.left as f32;
                let gy = physical_glyph.y as f32 - placement.top as f32;
                glyphs_to_draw.push((texture, gx, gy, w, h));
            }
        }

        if glyphs_to_draw.is_empty() {
            return Ok(());
        }

        glyphs_to_draw.sort_by_key(|&(tex, ..)| tex.0);

        unsafe {
            let gl = &self.gl_context;
            self.text_program.use_program(gl);

            let tex = self.create_gradient_texture(&brush_data).unwrap_or(None);

            self.text_program.bind_transform(gl, self.proj_transform);

            if let Some(t_id) = tex {
                // 如果启用了超大渐变，bind_brush_data里把 u_stop_data 绑定到了 TEXTURE1
                self.text_program.bind_brush_data(gl, &brush_data, 1);
                gl.active_texture(glow::TEXTURE1);
                gl.bind_texture(glow::TEXTURE_2D, Some(t_id));
            } else {
                self.text_program.bind_brush_data(gl, &brush_data, 0);
            }
            // 字体图集永远绑定在 TEXTURE0
            self.text_program.bind_texture(gl, 0);
            gl.active_texture(glow::TEXTURE0);

            gl.bind_vertex_array(Some(self.text_vao));

            let mut batch_vertices = Vec::new();

            for chunk in glyphs_to_draw.chunk_by(|a, b| a.0 == b.0) {
                let (tex, _, _, _, _) = chunk[0];

                // 确保有足够的预分配容量 (每个字符占用 24 个 float)
                batch_vertices.clear();
                batch_vertices.reserve(chunk.len() * 24);

                // 内层紧凑的循环，全速压入所有同属该贴图的顶点，没有多余的状态机逻辑
                for &(_, gx, gy, w, h) in chunk {
                    batch_vertices.extend_from_slice(&[
                        gx,
                        gy,
                        0.0,
                        0.0,
                        gx + w,
                        gy,
                        1.0,
                        0.0,
                        gx + w,
                        gy + h,
                        1.0,
                        1.0,
                        gx,
                        gy,
                        0.0,
                        0.0,
                        gx + w,
                        gy + h,
                        1.0,
                        1.0,
                        gx,
                        gy + h,
                        0.0,
                        1.0,
                    ]);
                }

                // 这个 Chunk 里的字符组装完毕，一次性发往显卡！
                gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.text_vbo));

                let vertex_bytes = slice::from_raw_parts(
                    batch_vertices.as_ptr() as *const u8,
                    batch_vertices.len() * size_of::<f32>(),
                );

                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_bytes, glow::DYNAMIC_DRAW);
                gl.draw_arrays(glow::TRIANGLES, 0, (batch_vertices.len() / 4) as i32);
            }

            if let Some(t_id) = tex {
                gl.delete_texture(t_id);
            }
            self.text_program.unbind(gl);
        }
        Ok(())
    }

    fn draw_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        stroke_width: f32,
        _options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        time_it!("draw_path");
        let brush_data = brush.to_shader_data();
        let geometry = self.tessellator.tessellate_stroke(path, stroke_width)?;

        unsafe {
            self.render_tessellated_geometry(&geometry, &brush_data)?;
        }
        Ok(())
    }

    fn fill_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        _options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        time_it!("fill_path");
        let brush_data = brush.to_shader_data();
        let geometry = self.tessellator.tessellate_fill(path)?;

        unsafe {
            self.render_tessellated_geometry(&geometry, &brush_data)?;
        }
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
        time_it!("draw_quad");
        let path = Path::from_rect(left, top, width, height);

        self.draw_path(&path, brush, border_width, options)
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
        time_it!("fill_quad");
        let path = if let Some(r) = corner_radius {
            Path::from_rounded_rect(left, top, width, height, r)
        } else {
            Path::from_rect(left, top, width, height)
        };

        self.fill_path(&path, brush, options)
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
        time_it!("blur_quad");
        let path = if corner_radius > 0.0 {
            Path::from_rounded_rect(left, top, width, height, corner_radius)
        } else {
            Path::from_rect(left, top, width, height)
        };
        self.blur_path(&path, blur_radius, transform)
    }

    fn blur_path(
        &mut self,
        path: &Path,
        blur_radius: f32,
        options_transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error> {
        time_it!("blur_path");
        if blur_radius <= 0.0 {
            return Ok(());
        }

        let geometry = self.tessellator.tessellate_fill(path)?;

        if geometry.vertices.is_empty() || geometry.indices.is_empty() {
            return Ok(());
        }

        let (win_w, win_h) = self.window_size;
        if win_w == 0 || win_h == 0 {
            return Ok(());
        }

        unsafe {
            let gl = &self.gl_context;

            // 准备两个 FBO 级纹理（全屏大小）进行 Ping-Pong 渲染
            let screen_texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(screen_texture));
            // 将整个屏幕当前的画面 Copy 下来作为底层模糊输入 (系统可能无 Alpha 通道，使用 RGB 默认 Alpha = 1.0)
            gl.copy_tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB,
                0,
                0,
                win_w as i32,
                win_h as i32,
                0,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            // 创建用于存放横向模糊结果的中间纹理和 FBO
            let pingpong_fbo = gl.create_framebuffer()?;
            let pingpong_texture = gl.create_texture_tex_image_2d(
                win_w as i32,
                win_h as i32,
                PixelUnpackData::Slice(None),
            )?;

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(pingpong_fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(pingpong_texture),
                0,
            );

            let scaled_radius = blur_radius * self.dpi_scale.0;

            // ================= Pass 1: Horizontal Blur =================
            // FBO 内的渲染不应该叠加背景混合，而是完全替换
            gl.disable(glow::BLEND);

            self.blur_program.use_program(gl);
            // PASS 1 的矩阵使用全屏直出投影，因为我们只做缓冲复制
            self.blur_program.bind_transform(gl, Transform2D::IDENTITY);
            self.blur_program.bind_texture(gl, 0);
            self.blur_program
                .bind_resolution(gl, win_w as f32, win_h as f32);
            self.blur_program.bind_blur_radius(gl, scaled_radius);
            self.blur_program.bind_direction(gl, 1.0, 0.0); // 水平

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(screen_texture));

            // 构建全屏四边形用于拷贝 (stride=8 也就是 2 个 floats, [x, y])
            let fullscreen_vertices: [f32; 12] = [
                -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
            ];

            self.render_arrays(&fullscreen_vertices, glow::TRIANGLES, 6)?;

            // ================= Pass 2: Vertical Blur to Final Shape =================
            // 切回默认帧缓冲 (也就是输出到屏幕) 并开回混合
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.enable(glow::BLEND);

            self.blur_program.bind_transform(gl, self.proj_transform);
            self.blur_program.bind_direction(gl, 0.0, 1.0); // 垂直

            // 输入纹理变为从 FBO 生成的刚刚只经历横向模糊的 Pingpong
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(pingpong_texture));

            // 这次我们用真正的路径网格来约束输出结果和裁剪！
            self.render_elements(&geometry.vertices, &geometry.indices)?;

            // 清理临时对象
            gl.delete_texture(screen_texture);
            gl.delete_texture(pingpong_texture);
            gl.delete_framebuffer(pingpong_fbo);

            self.blur_program.unbind(gl);
        }

        Ok(())
    }

    fn push_clip(&mut self, _rect: (f32, f32, f32, f32)) -> Result<(), Self::Error> {
        // We will increment the stencil mask value inside the clip region later
        Ok(())
    }

    fn push_rounded_clip(
        &mut self,
        _rect: (f32, f32, f32, f32),
        _radius: f32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn push_path_clip(&mut self, _path: &Path) -> Result<(), Self::Error> {
        Ok(())
    }

    fn pop_clip(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_clip_depth(&mut self) -> Result<u32, Self::Error> {
        todo!()
    }

    fn suspend_clip(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn resume_clip(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn push_transform(&mut self, transform: &Transform2D) -> Result<(), Self::Error> {
        time_it!("push_transform");
        Ok(())
    }

    fn pop_transform(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        time_it!("pop_transform");
        if let Some(depth) = target_depth {
            if depth < self.transform_stack.len() as u32 {
                self.transform_stack.truncate(depth as usize);
            }
        } else if self.transform_stack.len() > 1 {
            self.transform_stack.pop();
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
        Ok(vec![])
    }
}

mod utils {
    pub struct Timer {
        name: &'static str,
        start: std::time::Instant,
    }

    impl Timer {
        pub fn new(name: &'static str) -> Self {
            Self {
                name,
                start: std::time::Instant::now(),
            }
        }
    }

    impl Drop for Timer {
        fn drop(&mut self) {
            println!(
                "[GlRenderer Flow] {} took {:?}",
                self.name,
                self.start.elapsed()
            );
        }
    }
}

impl GlRenderer {
    unsafe fn create_gradient_texture(
        &self,
        brush_data: &crate::handle::GlGradientData,
    ) -> Result<Option<glow::Texture>, GlError> {
        let num_pixels = 256;
        let pixels = brush_data.get_texture_pixels(num_pixels);
        if pixels.is_empty() {
            return Ok(None);
        }
        let gl = &self.gl_context;
        let tex = gl.create_texture()?;
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA32F as i32,
            num_pixels as i32,
            1,
            0,
            glow::RGBA,
            glow::FLOAT,
            PixelUnpackData::Slice(Some(std::slice::from_raw_parts(
                pixels.as_ptr() as *const u8,
                pixels.len() * std::mem::size_of::<f32>(),
            ))),
        );
        Ok(Some(tex))
    }

    #[inline]
    unsafe fn render_tessellated_geometry(
        &self,
        geometry: &lyon_tessellation::VertexBuffers<Vertex, u32>,
        brush_data: &crate::handle::GlGradientData,
    ) -> Result<(), GlError> {
        if geometry.vertices.is_empty() || geometry.indices.is_empty() {
            return Ok(());
        }

        let gl = &self.gl_context;
        self.color_program.use_program(gl);
        self.color_program.bind_transform(gl, self.proj_transform);

        let tex_opt = self.create_gradient_texture(brush_data)?;
        if let Some(tex) = tex_opt {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        }

        self.color_program.bind_brush_data(gl, brush_data, 0);

        self.render_elements(&geometry.vertices, &geometry.indices)?;

        if let Some(tex) = tex_opt {
            gl.delete_texture(tex);
        }
        self.color_program.unbind(gl);

        Ok(())
    }
    #[inline]
    unsafe fn render_elements<V>(&self, vertices: &[V], indices: &[u32]) -> Result<(), GlError> {
        let gl = &self.gl_context;
        gl.bind_vertex_array(Some(self.color_vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.color_vbo));

        let vertex_bytes = slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * size_of::<V>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_bytes, glow::DYNAMIC_DRAW);

        let ebo = gl.create_buffer()?;
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

        let index_bytes = slice::from_raw_parts(
            indices.as_ptr() as *const u8,
            indices.len() * size_of::<u32>(),
        );
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, index_bytes, glow::DYNAMIC_DRAW);

        gl.draw_elements(glow::TRIANGLES, indices.len() as i32, glow::UNSIGNED_INT, 0);

        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        gl.delete_buffer(ebo);
        Ok(())
    }

    #[inline]
    unsafe fn render_arrays<V>(
        &self,
        vertices: &[V],
        mode: u32,
        count: i32,
    ) -> Result<(), GlError> {
        let gl = &self.gl_context;
        gl.bind_vertex_array(Some(self.color_vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.color_vbo));

        let vertex_bytes = std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * std::mem::size_of::<V>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_bytes, glow::DYNAMIC_DRAW);

        gl.draw_arrays(mode, 0, count);

        gl.bind_vertex_array(None);
        Ok(())
    }

    fn prepare_text_buffer(
        font_system: &mut FontSystem,
        text: &str,
        text_format: &GlTextFormatHandle,
        width: f32,
        height: f32,
        dpi_x: f32,
        dpi_y: f32,
    ) -> cosmic_text::Buffer {
        let font_size = text_format.font_size * dpi_y;
        let phys_width = width * dpi_x;
        let phys_height = height * dpi_y;

        let mut buffer = text_format.create_buffer(font_system, font_size);
        buffer.set_size(
            font_system,
            if width > 0.0 { Some(phys_width) } else { None },
            if height > 0.0 {
                Some(phys_height)
            } else {
                None
            },
        );

        text_format.apply_wrap(font_system, &mut buffer);
        text_format.apply_text_with_trimming(
            font_system,
            &mut buffer,
            text,
            width,
            height,
            phys_width,
            phys_height,
        );

        buffer
    }

    pub fn draw_texture(
        &self,
        draw_x: f32,
        draw_y: f32,
        draw_w: f32,
        draw_h: f32,
        texture: NativeTexture,
        opacity: Option<f32>,
    ) {
        let vertices: [f32; 24] = [
            draw_x,
            draw_y,
            0.0,
            0.0, // 左上角点 (在原图上的抓取坐标为 0,0)
            draw_x + draw_w,
            draw_y,
            1.0,
            0.0, // 右上角点 (原图 U 拉满 = 1，V 未动 = 0)
            draw_x + draw_w,
            draw_y + draw_h,
            1.0,
            1.0, // 右下角点 (两边拉满)
            draw_x,
            draw_y,
            0.0,
            0.0, // 第二个三角形的起点（左上）
            draw_x + draw_w,
            draw_y + draw_h,
            1.0,
            1.0, // 结尾点（右下）
            draw_x,
            draw_y + draw_h,
            0.0,
            1.0, // 左下角点
        ];

        unsafe {
            let gl = &self.gl_context;

            self.texture_program.use_program(gl);
            self.texture_program.bind_transform(gl, self.proj_transform);
            self.texture_program.bind_texture(gl, 0);
            self.texture_program.bind_opacity(gl, opacity);

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.bind_vertex_array(Some(self.text_vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.text_vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                vertices.align_to::<u8>().1,
                glow::DYNAMIC_DRAW,
            );

            gl.draw_arrays(glow::TRIANGLES, 0, 6);

            // 恢复环境解绑
            gl.bind_vertex_array(None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.use_program(None);
        }
    }
}
