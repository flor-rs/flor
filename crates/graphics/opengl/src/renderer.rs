use crate::error::GlError;
use crate::handle::GlImageHandle;
use crate::handle::GlSurfaceId;
use crate::handle::{GlBrushHandle, GlTextFormatHandle};
use cosmic_text::{CacheKey, FontSystem, Placement, SwashCache};
use flor_base::graphics::{
    Gradient, HitTestResult, ImageDrawOptions, ParagraphAlignment, Path, PathDrawOptions, Render,
    RenderContext, ScaleMode, Shadow, SurfaceDrawOptions, TextDrawOptions,
};
use flor_base::types::{Color, Transform2D};
use glow::{HasContext, NativeTexture, PixelUnpackData};
use parking_lot::Mutex;
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

pub mod context;

pub(crate) static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

macro_rules! time_it {
    ($name:literal) => {
        let _timer = crate::renderer::utils::Timer::new($name);
    };
}

// #[derive(Debug)] removed for manual implementation
pub struct GlRenderer {
    display_context: NativeDisplayContext,
    pub gl_context: Arc<GlContext>,

    // Scale and Viewport
    // dpi 缩放倍率，这里是倍率
    pub dpi_scale: (f32, f32),
    pub window_size: (u32, u32),

    pub proj_transform: Transform2D,
    // Transform Stack
    pub transform_stack: Vec<Transform2D>,

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

    // MSAA States
    pub msaa_fbo: Option<glow::Framebuffer>,
    pub msaa_color: Option<glow::Renderbuffer>,
    pub msaa_depth_stencil: Option<glow::Renderbuffer>,

    pub image_texture_cache: HashMap<(u64, usize), glow::Texture>,
    pub next_image_id: u64,
    pub current_surface: Option<GlSurfaceId>,
    pub clip_stack_depth: u32,
    pub saved_transform_stack: Vec<Transform2D>,
    pub saved_clip_stack_depth: u32,
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
        let gl_context = Arc::new(GlContext::from_context(display_context.get_gl_context()));
        unsafe {
            display_context.set_v_sync(wait_v_sync);
            gl_context.viewport(0, 0, width as i32, height as i32);

            gl_context.enable(glow::BLEND);
            gl_context.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );
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

            msaa_fbo: None,
            msaa_color: None,
            msaa_depth_stencil: None,

            image_texture_cache: HashMap::new(),
            next_image_id: 1,
            current_surface: None,
            clip_stack_depth: 0,
            saved_transform_stack: vec![],
            saved_clip_stack_depth: 0,
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
        let gl = &self.gl_context;
        unsafe {
            let (dpi_x, dpi_y) = self.dpi_scale;
            let phys_w = (self.window_size.0 as f32 * dpi_x).ceil() as i32;
            let phys_h = (self.window_size.1 as f32 * dpi_y).ceil() as i32;

            if phys_w > 0 && phys_h > 0 && self.msaa_fbo.is_some() {
                // 1. Resolve MSAA directly to screen
                gl.bind_framebuffer(glow::READ_FRAMEBUFFER, self.msaa_fbo);
                gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
                gl.blit_framebuffer(
                    0,
                    0,
                    phys_w,
                    phys_h,
                    0,
                    0,
                    phys_w,
                    phys_h,
                    glow::COLOR_BUFFER_BIT,
                    glow::NEAREST,
                );
            }
        }
        self.display_context.present();
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        time_it!("clear");
        unsafe {
            // Ensure we are targeting the msaa_fbo if no surface is bound
            if self.current_surface.is_none() {
                self.gl_context
                    .bind_framebuffer(glow::FRAMEBUFFER, self.msaa_fbo);
            }
            self.gl_context.clear_color(
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                color.a as f32 / 255.0,
            );
            self.gl_context.clear_stencil(0);
            self.gl_context
                .clear(glow::COLOR_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
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
            let gl = &self.gl_context;
            let proj_transform = Transform2D::ortho(width as f32, height as f32);
            self.proj_transform = proj_transform;

            let (dpi_x, dpi_y) = self.dpi_scale;
            let phys_w = (width as f32 * dpi_x).ceil() as i32;
            let phys_h = (height as f32 * dpi_y).ceil() as i32;

            gl.viewport(0, 0, phys_w, phys_h);

            // 1. Release old resources
            if let Some(fbo) = self.msaa_fbo.take() {
                gl.delete_framebuffer(fbo);
            }
            if let Some(rbo) = self.msaa_color.take() {
                gl.delete_renderbuffer(rbo);
            }
            if let Some(rbo) = self.msaa_depth_stencil.take() {
                gl.delete_renderbuffer(rbo);
            }

            if phys_w > 0 && phys_h > 0 {
                let samples = 4;

                // --- MSAA FBO ---
                let msaa_fbo = gl.create_framebuffer()?;
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(msaa_fbo));

                let msaa_color = gl.create_renderbuffer()?;
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(msaa_color));
                gl.renderbuffer_storage_multisample(
                    glow::RENDERBUFFER,
                    samples,
                    glow::RGBA8,
                    phys_w,
                    phys_h,
                );
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::RENDERBUFFER,
                    Some(msaa_color),
                );

                let msaa_depth_stencil = gl.create_renderbuffer()?;
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(msaa_depth_stencil));
                gl.renderbuffer_storage_multisample(
                    glow::RENDERBUFFER,
                    samples,
                    glow::DEPTH24_STENCIL8,
                    phys_w,
                    phys_h,
                );
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_STENCIL_ATTACHMENT,
                    glow::RENDERBUFFER,
                    Some(msaa_depth_stencil),
                );

                if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                    log::warn!("MSAA Framebuffer is not complete!");
                }

                self.msaa_fbo = Some(msaa_fbo);
                self.msaa_color = Some(msaa_color);
                self.msaa_depth_stencil = Some(msaa_depth_stencil);

                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            }
        }
        Ok(())
    }

    fn set_scale_factor(&mut self, dpi_x: f32, dpi_y: f32) -> Result<(), Self::Error> {
        self.dpi_scale = (dpi_x, dpi_y);
        let (w, h) = self.window_size;
        self.update_window_size(w, h)?;
        Ok(())
    }

    fn create_surface(&mut self, width: u32, height: u32) -> Result<Self::SurfaceId, Self::Error> {
        let (dpi_x, dpi_y) = self.dpi_scale;
        let physical_w = (width as f32 * dpi_x).ceil() as i32;
        let physical_h = (height as f32 * dpi_y).ceil() as i32;

        unsafe {
            let gl = &self.gl_context;

            // 使用封装好的上下文方法创建纹理，它已包含了 tex_image_2d 和 filter 配置
            let texture = gl.create_texture_tex_image_2d(
                physical_w,
                physical_h,
                PixelUnpackData::Slice(None),
            )?;

            // 创建 FBO
            let fbo = gl.create_framebuffer()?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );

            // Add Depth/Stencil Renderbuffer
            let rbo = gl.create_renderbuffer()?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
            gl.renderbuffer_storage(
                glow::RENDERBUFFER,
                glow::DEPTH24_STENCIL8,
                physical_w,
                physical_h,
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_STENCIL_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(rbo),
            );

            // 检查 FBO 状态
            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                return Err(GlError::CustomError(
                    "Framebuffer is not complete".to_string(),
                ));
            }

            // 恢复默认状态
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(GlSurfaceId {
                width,
                height,
                inner: Arc::new(crate::handle::SurfaceInner {
                    gl_context: self.gl_context.clone(),
                    texture,
                    fbo,
                    rbo: Some(rbo),
                }),
            })
        }
    }

    fn set_render_target(&mut self, surface_id: &Self::SurfaceId) -> Result<(), Self::Error> {
        let (dpi_x, dpi_y) = self.dpi_scale;
        let physical_w = (surface_id.width as f32 * dpi_x).ceil() as i32;
        let physical_h = (surface_id.height as f32 * dpi_y).ceil() as i32;

        unsafe {
            self.gl_context
                .bind_framebuffer(glow::FRAMEBUFFER, Some(surface_id.inner.fbo));
            self.gl_context.viewport(0, 0, physical_w, physical_h);
            self.proj_transform =
                Transform2D::ortho(surface_id.width as f32, surface_id.height as f32);
        }

        self.saved_transform_stack = std::mem::take(&mut self.transform_stack);
        self.saved_clip_stack_depth = self.clip_stack_depth;
        self.clip_stack_depth = 0;

        self.current_surface = Some(surface_id.clone());
        Ok(())
    }

    fn reset_render_target(&mut self) -> Result<(), Self::Error> {
        let (win_w, win_h) = self.window_size;
        unsafe {
            let (dpi_x, dpi_y) = self.dpi_scale;
            let phys_w = (win_w as f32 * dpi_x).ceil() as i32;
            let phys_h = (win_h as f32 * dpi_y).ceil() as i32;

            self.gl_context
                .bind_framebuffer(glow::FRAMEBUFFER, self.msaa_fbo);
            self.gl_context.viewport(0, 0, phys_w, phys_h);
            self.proj_transform = Transform2D::ortho(win_w as f32, win_h as f32);
        }

        self.transform_stack = std::mem::take(&mut self.saved_transform_stack);
        self.clip_stack_depth = self.saved_clip_stack_depth;

        // Restore stencil state
        unsafe {
            if self.clip_stack_depth == 0 {
                self.gl_context.disable(glow::STENCIL_TEST);
            } else {
                self.gl_context.enable(glow::STENCIL_TEST);
                self.gl_context
                    .stencil_func(glow::LEQUAL, self.clip_stack_depth as i32, 0xFF);
                self.gl_context
                    .stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
            }
        }

        self.current_surface = None;
        Ok(())
    }

    fn create_image_from_bytes(&mut self, bytes: &[u8]) -> Result<Self::ImageHandle, Self::Error> {
        time_it!("create_image_from_bytes");
        let id = self.next_image_id;
        self.next_image_id += 1;

        Ok(GlImageHandle::new(id, bytes, self.gl_context.clone())?)
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
            cache: Arc::new(crate::handle::ImageCacheInner {
                gl_context: self.gl_context.clone(),
                textures: Mutex::new(Vec::new()),
            }),
        })
    }

    #[cfg(feature = "svg")]
    fn create_svg(&mut self, bytes: &[u8]) -> Result<Self::SvgHandle, Self::Error> {
        time_it!("create_svg");
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(bytes, &opt)
            .map_err(|e| GlError::CustomError(format!("SVG parse error: {:?}", e)))?;

        Ok(GlSvgHandle {
            cache: Arc::new(crate::handle::SvgCacheInner {
                gl_context: self.gl_context.clone(),
                texture: Mutex::new(None),
            }),
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
        let local_transform = options.and_then(|o| o.transform.as_ref());

        let target_w = width.unwrap_or(handle.width as f32);
        let target_h = height.unwrap_or(handle.height as f32);

        if target_w <= 0.0 || target_h <= 0.0 || handle.width == 0 || handle.height == 0 {
            return Ok(());
        }

        let scale_mode = options
            .and_then(|o| o.scale_mode)
            .unwrap_or(ScaleMode::None);

        let img_w = handle.width as f32;
        let img_h = handle.height as f32;

        let (draw_x, draw_y, draw_w, draw_h) =
            scale_mode.calc_draw_rect(x, y, target_w, target_h, img_w, img_h);

        let frame_index = options
            .and_then(|o| o.frame_index)
            .unwrap_or(0)
            .min(handle.frames.len().saturating_sub(1));

        let texture = {
            let mut cache = handle.cache.textures.lock();
            if cache.is_empty() {
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
        let shadow = options.and_then(|o| o.shadow.as_ref());

        let need_clip = matches!(scale_mode, ScaleMode::Cover);
        if need_clip {
            self.push_clip((x, y, target_w, target_h))?;
        }

        self.draw_texture(
            draw_x,
            draw_y,
            draw_w,
            draw_h,
            texture,
            opacity,
            None,
            local_transform,
            shadow,
        )?;

        if need_clip {
            self.pop_clip(None)?;
        }

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
        let local_transform = options.and_then(|o| o.transform.as_ref());

        let scale_mode = options
            .and_then(|o| o.scale_mode)
            .unwrap_or(ScaleMode::None);

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

        let physical_w = (draw_w * self.dpi_scale.0).ceil() as u32;
        let physical_h = (draw_h * self.dpi_scale.1).ceil() as u32;

        let texture = {
            let mut option_texture = handle.cache.texture.lock();

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

                    let new_texture = self.gl_context.create_texture_tex_image_2d(
                        physical_w as i32,
                        physical_h as i32,
                        PixelUnpackData::Slice(Some(pixmap.data())),
                    )?;

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
        let shadow = options.and_then(|o| o.shadow.as_ref());

        let need_clip = matches!(scale_mode, ScaleMode::Cover);
        if need_clip {
            self.push_clip((x, y, target_w, target_h))?;
        }

        self.draw_texture(
            draw_x,
            draw_y,
            draw_w,
            draw_h,
            texture,
            opacity,
            None,
            local_transform,
            shadow,
        )?;

        if need_clip {
            self.pop_clip(None)?;
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
        time_it!("draw_surface");
        let local_transform = options.and_then(|o| o.transform.as_ref());

        let target_w = width.unwrap_or(handle.width as f32);
        let target_h = height.unwrap_or(handle.height as f32);

        if target_w <= 0.0 || target_h <= 0.0 || handle.width == 0 || handle.height == 0 {
            return Ok(());
        }

        let scale_mode = options
            .and_then(|o| o.scale_mode)
            .unwrap_or(ScaleMode::None);

        let img_w = handle.width as f32;
        let img_h = handle.height as f32;

        let (draw_x, draw_y, draw_w, draw_h) =
            scale_mode.calc_draw_rect(x, y, target_w, target_h, img_w, img_h);

        let opacity = options.and_then(|o| o.opacity);
        let shadow = options.and_then(|o| o.shadow.as_ref());

        let need_clip = matches!(scale_mode, ScaleMode::Cover);
        if need_clip {
            self.push_clip((x, y, target_w, target_h))?;
        }

        self.draw_texture(
            draw_x,
            draw_y,
            draw_w,
            draw_h,
            handle.inner.texture,
            opacity,
            None,
            local_transform,
            shadow,
        )?;

        if need_clip {
            self.pop_clip(None)?;
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
        options: Option<&TextDrawOptions>,
    ) -> Result<(), Self::Error> {
        time_it!("draw_text");
        let local_transform = options.and_then(|o| o.transform.as_ref());

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
                    std::collections::hash_map::Entry::Occupied(o) => o.get().clone(),
                    std::collections::hash_map::Entry::Vacant(v) => {
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
                                cosmic_text::SwashContent::Mask => {
                                    for &a in &image.data {
                                        rgba_data.push(255);
                                        rgba_data.push(255);
                                        rgba_data.push(255);
                                        rgba_data.push(a);
                                    }
                                }
                                cosmic_text::SwashContent::SubpixelMask => {
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
                                cosmic_text::SwashContent::Color => {
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

                let w = placement.width as f32 / dpi_x;
                let h = placement.height as f32 / dpi_y;
                let gx = (physical_glyph.x as f32 + placement.left as f32) / dpi_x;
                let gy = (physical_glyph.y as f32 - placement.top as f32) / dpi_y;
                glyphs_to_draw.push((texture, gx, gy, w, h));
            }
        }

        let shadow = options.and_then(|o| o.shadow.as_ref());
        let bounds = (left, top, width, height);

        unsafe {
            self.render_text_glyphs(
                &mut glyphs_to_draw,
                &brush_data,
                local_transform,
                shadow,
                bounds,
            )?;
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
        time_it!("draw_path");
        let local_transform = options.and_then(|o| o.transform.as_ref());

        let brush_data = brush.to_shader_data();
        let geometry = self.tessellator.tessellate_stroke(
            path,
            stroke_width,
            self.dpi_scale.0,
            self.dpi_scale.1,
        )?;

        let shadow = options.and_then(|o| o.shadow.as_ref());
        let (min_x, min_y, max_x, max_y) = path.get_bounds();
        let bounds = (min_x, min_y, max_x - min_x, max_y - min_y);

        unsafe {
            self.render_tessellated_geometry(
                &geometry,
                &brush_data,
                local_transform,
                shadow,
                Some(path),
                bounds,
            )?;
        }

        Ok(())
    }

    fn fill_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        time_it!("fill_path");
        let local_transform = options.and_then(|o| o.transform.as_ref());

        let brush_data = brush.to_shader_data();
        let geometry =
            self.tessellator
                .tessellate_fill(path, self.dpi_scale.0, self.dpi_scale.1)?;

        let shadow = options.and_then(|o| o.shadow.as_ref());
        let (min_x, min_y, max_x, max_y) = path.get_bounds();
        let bounds = (min_x, min_y, max_x - min_x, max_y - min_y);

        unsafe {
            self.render_tessellated_geometry(
                &geometry,
                &brush_data,
                local_transform,
                shadow,
                Some(path),
                bounds,
            )?;
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

        let geometry =
            self.tessellator
                .tessellate_fill(path, self.dpi_scale.0, self.dpi_scale.1)?;

        if geometry.vertices.is_empty() || geometry.indices.is_empty() {
            return Ok(());
        }

        let (fbo_w, fbo_h) = self.get_fbo_size();

        if fbo_w <= 0 || fbo_h <= 0 {
            return Ok(());
        }

        unsafe {
            let gl = &self.gl_context;

            // 准备两个 FBO 级纹理（全屏大小）进行 Ping-Pong 渲染
            let screen_texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(screen_texture));
            // 将整个屏幕当前的画面 Copy 下来作为底层模糊输入 (系统可能无 Alpha 通道，使用 RGB 默认 Alpha = 1.0)
            gl.copy_tex_image_2d(glow::TEXTURE_2D, 0, glow::RGB, 0, 0, fbo_w, fbo_h, 0);

            // 创建用于存放横向模糊结果的中间纹理和 FBO
            let pingpong_fbo = gl.create_framebuffer()?;
            let pingpong_texture =
                gl.create_texture_tex_image_2d(fbo_w, fbo_h, PixelUnpackData::Slice(None))?;

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
                .bind_resolution(gl, fbo_w as f32, fbo_h as f32);
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
            // 切回目标帧缓冲并开回混合
            if let Some(surface) = &self.current_surface {
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(surface.inner.fbo));
            } else {
                gl.bind_framebuffer(glow::FRAMEBUFFER, self.msaa_fbo);
            }
            gl.enable(glow::BLEND);

            self.blur_program
                .bind_transform(gl, self.get_final_transform(options_transform));
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

    fn push_clip(&mut self, rect: (f32, f32, f32, f32)) -> Result<(), Self::Error> {
        let path = Path::from_rect(rect.0, rect.1, rect.2, rect.3);
        self.push_path_clip(&path)
    }

    fn push_rounded_clip(
        &mut self,
        rect: (f32, f32, f32, f32),
        radius: f32,
    ) -> Result<(), Self::Error> {
        let path = Path::from_rounded_rect(rect.0, rect.1, rect.2, rect.3, radius);
        self.push_path_clip(&path)
    }

    fn push_path_clip(&mut self, path: &Path) -> Result<(), Self::Error> {
        time_it!("push_path_clip");
        let brush = self.create_solid_color_brush(Color::rgba(0, 0, 0, 0), None)?;

        unsafe {
            if self.clip_stack_depth == 0 {
                self.gl_context.enable(glow::STENCIL_TEST);
            }

            self.gl_context.color_mask(false, false, false, false);
            self.gl_context.depth_mask(false);

            self.gl_context
                .stencil_func(glow::LEQUAL, self.clip_stack_depth as i32, 0xFF);
            self.gl_context
                .stencil_op(glow::KEEP, glow::KEEP, glow::INCR);

            self.fill_path(path, &brush, None)?;

            self.clip_stack_depth += 1;

            self.gl_context.color_mask(true, true, true, true);
            self.gl_context.depth_mask(true);
            self.gl_context
                .stencil_func(glow::LEQUAL, self.clip_stack_depth as i32, 0xFF);
            self.gl_context
                .stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
        }

        Ok(())
    }

    fn pop_clip(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        time_it!("pop_clip");
        let target = target_depth.unwrap_or_else(|| self.clip_stack_depth.saturating_sub(1));

        if self.clip_stack_depth <= target {
            return Ok(());
        }

        let brush = self.create_solid_color_brush(Color::rgba(0, 0, 0, 0), None)?;
        let brush_data = brush.to_shader_data();

        unsafe {
            self.gl_context.color_mask(false, false, false, false);
            self.gl_context.depth_mask(false);

            let gl = &self.gl_context;
            self.color_program.use_program(gl);
            self.color_program.bind_transform(gl, Transform2D::IDENTITY);
            self.color_program.bind_brush_data(gl, &brush_data, 0);

            let fullscreen_vertices: [f32; 12] = [
                -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
            ];

            while self.clip_stack_depth > target {
                self.gl_context
                    .stencil_func(glow::LEQUAL, self.clip_stack_depth as i32, 0xFF);
                self.gl_context
                    .stencil_op(glow::KEEP, glow::KEEP, glow::DECR);

                self.render_arrays(&fullscreen_vertices, glow::TRIANGLES, 6)?;

                self.clip_stack_depth -= 1;
            }

            self.color_program.unbind(gl);

            self.gl_context.color_mask(true, true, true, true);
            self.gl_context.depth_mask(true);

            if self.clip_stack_depth == 0 {
                self.gl_context.disable(glow::STENCIL_TEST);
            } else {
                self.gl_context
                    .stencil_func(glow::LEQUAL, self.clip_stack_depth as i32, 0xFF);
                self.gl_context
                    .stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
            }
        }

        Ok(())
    }

    fn get_clip_depth(&mut self) -> Result<u32, Self::Error> {
        Ok(self.clip_stack_depth)
    }

    fn suspend_clip(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.gl_context.disable(glow::STENCIL_TEST);
        }
        Ok(())
    }

    fn resume_clip(&mut self) -> Result<(), Self::Error> {
        if self.clip_stack_depth > 0 {
            unsafe {
                self.gl_context.enable(glow::STENCIL_TEST);
                self.gl_context
                    .stencil_func(glow::LEQUAL, self.clip_stack_depth as i32, 0xFF);
                self.gl_context
                    .stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
            }
        }
        Ok(())
    }

    fn push_transform(&mut self, transform: &Transform2D) -> Result<(), Self::Error> {
        time_it!("push_transform");
        let current = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(Transform2D::IDENTITY);
        self.transform_stack.push(current.multiply(transform));
        Ok(())
    }

    fn pop_transform(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        time_it!("pop_transform");
        if let Some(depth) = target_depth {
            if depth < self.transform_stack.len() as u32 {
                self.transform_stack.truncate(depth as usize);
            }
        } else if !self.transform_stack.is_empty() {
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
    #[inline]
    fn get_fbo_size(&self) -> (i32, i32) {
        if let Some(surface) = &self.current_surface {
            let (dpi_x, dpi_y) = self.dpi_scale;
            (
                (surface.width as f32 * dpi_x).ceil() as i32,
                (surface.height as f32 * dpi_y).ceil() as i32,
            )
        } else {
            (self.window_size.0 as i32, self.window_size.1 as i32)
        }
    }

    #[inline]
    fn apply_shadow_clip(
        &mut self,
        shadow: &Shadow,
        bounds: (f32, f32, f32, f32),
        clip_path: Option<&Path>,
    ) -> Result<(), GlError> {
        if shadow.inset {
            if let Some(p) = clip_path {
                self.push_path_clip(p)?;
            } else if bounds.2 > 0.0 && bounds.3 > 0.0 {
                self.push_clip(bounds)?;
            }
        }
        Ok(())
    }

    unsafe fn render_shadow<F>(
        &mut self,
        shadow: &Shadow,
        bounds: (f32, f32, f32, f32), // x, y, w, h
        clip_path: Option<&Path>,     // for clipping inset
        base_transform: Option<&Transform2D>,
        draw_mask: F,
    ) -> Result<(), GlError>
    where
        F: FnOnce(&mut Self, Option<&Transform2D>) -> Result<(), GlError>,
    {
        let (x, y, w, h) = bounds;

        // 1. Calculate transforms
        let spread = shadow.spread;
        let sx = if w > 0.0 { (w + 2.0 * spread) / w } else { 1.0 };
        let sy = if h > 0.0 { (h + 2.0 * spread) / h } else { 1.0 };
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;

        let spread_transform = Transform2D::scale_at(sx, sy, cx, cy);
        let offset_transform = Transform2D::translation(shadow.offset_x, shadow.offset_y);
        let shadow_transform = spread_transform.multiply(&offset_transform);

        let combined_transform = if let Some(base) = base_transform {
            base.multiply(&shadow_transform)
        } else {
            shadow_transform
        };

        if shadow.blur_radius <= 0.0 {
            self.apply_shadow_clip(shadow, (x, y, w, h), clip_path)?;

            draw_mask(self, Some(&combined_transform))?;

            if shadow.inset {
                self.gl_context.blend_func_separate(
                    glow::SRC_ALPHA,
                    glow::ONE_MINUS_SRC_ALPHA,
                    glow::ONE,
                    glow::ONE_MINUS_SRC_ALPHA,
                );
                self.pop_clip(None)?;
            }
            return Ok(());
        }

        let (fbo_w, fbo_h) = self.get_fbo_size();

        // --- FBO Blur Path ---
        let mask_texture = self.gl_context.create_texture_tex_image_2d(
            fbo_w,
            fbo_h,
            PixelUnpackData::Slice(None),
        )?;
        let mask_fbo = self.gl_context.create_framebuffer()?;
        self.gl_context
            .bind_framebuffer(glow::FRAMEBUFFER, Some(mask_fbo));
        self.gl_context.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(mask_texture),
            0,
        );
        self.gl_context.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl_context.clear(glow::COLOR_BUFFER_BIT);

        if shadow.inset {
            self.gl_context.clear_color(
                shadow.color.r as f32 / 255.0,
                shadow.color.g as f32 / 255.0,
                shadow.color.b as f32 / 255.0,
                shadow.color.a as f32 / 255.0,
            );
            self.gl_context.clear(glow::COLOR_BUFFER_BIT);
            self.gl_context.blend_func_separate(
                glow::ZERO,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ZERO,
                glow::ONE_MINUS_SRC_ALPHA,
            );
        }

        draw_mask(self, Some(&combined_transform))?;

        if shadow.inset {
            self.gl_context.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );
        }

        let pingpong_texture = self.gl_context.create_texture_tex_image_2d(
            fbo_w,
            fbo_h,
            PixelUnpackData::Slice(None),
        )?;
        let pingpong_fbo = self.gl_context.create_framebuffer()?;
        self.gl_context
            .bind_framebuffer(glow::FRAMEBUFFER, Some(pingpong_fbo));
        self.gl_context.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(pingpong_texture),
            0,
        );

        let scaled_radius = shadow.blur_radius * self.dpi_scale.0;

        self.gl_context.disable(glow::BLEND);
        self.blur_program.use_program(&self.gl_context);
        self.blur_program
            .bind_transform(&self.gl_context, Transform2D::IDENTITY);
        self.blur_program
            .bind_resolution(&self.gl_context, fbo_w as f32, fbo_h as f32);
        self.blur_program
            .bind_blur_radius(&self.gl_context, scaled_radius);
        self.blur_program.bind_texture(&self.gl_context, 0);

        self.blur_program.bind_direction(&self.gl_context, 1.0, 0.0);
        self.gl_context.active_texture(glow::TEXTURE0);
        self.gl_context
            .bind_texture(glow::TEXTURE_2D, Some(mask_texture));
        let fullscreen_vertices: [f32; 12] = [
            -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
        ];
        self.render_arrays(&fullscreen_vertices, glow::TRIANGLES, 6)?;

        if let Some(surface) = &self.current_surface {
            self.gl_context
                .bind_framebuffer(glow::FRAMEBUFFER, Some(surface.inner.fbo));
        } else {
            self.gl_context
                .bind_framebuffer(glow::FRAMEBUFFER, self.msaa_fbo);
        }
        self.gl_context.enable(glow::BLEND);
        // Pingpong texture has premultiplied alpha, so we must use ONE for source color
        self.gl_context.blend_func_separate(
            glow::ONE,
            glow::ONE_MINUS_SRC_ALPHA,
            glow::ONE,
            glow::ONE_MINUS_SRC_ALPHA,
        );

        if shadow.inset {
            if let Some(p) = clip_path {
                self.push_path_clip(p)?;
            } else if w > 0.0 && h > 0.0 {
                self.push_clip((x, y, w, h))?;
            }
        }

        self.blur_program.bind_direction(&self.gl_context, 0.0, 1.0);
        self.gl_context
            .bind_texture(glow::TEXTURE_2D, Some(pingpong_texture));
        self.render_arrays(&fullscreen_vertices, glow::TRIANGLES, 6)?;

        // Restore normal blend func
        self.gl_context.blend_func_separate(
            glow::SRC_ALPHA,
            glow::ONE_MINUS_SRC_ALPHA,
            glow::ONE,
            glow::ONE_MINUS_SRC_ALPHA,
        );

        if shadow.inset {
            self.pop_clip(None)?;
        }

        self.gl_context.delete_texture(mask_texture);
        self.gl_context.delete_framebuffer(mask_fbo);
        self.gl_context.delete_texture(pingpong_texture);
        self.gl_context.delete_framebuffer(pingpong_fbo);
        self.blur_program.unbind(&self.gl_context);

        Ok(())
    }

    unsafe fn render_text_glyphs(
        &mut self,
        glyphs_to_draw: &mut Vec<(glow::Texture, f32, f32, f32, f32)>,
        brush_data: &crate::handle::GlGradientData,
        local_transform: Option<&Transform2D>,
        shadow: Option<&Shadow>,
        bounds: (f32, f32, f32, f32),
    ) -> Result<(), GlError> {
        if glyphs_to_draw.is_empty() {
            return Ok(());
        }

        if let Some(shadow) = shadow {
            self.render_shadow(shadow, bounds, None, local_transform, |this, shadow_t| {
                let shadow_brush = this.create_solid_color_brush(shadow.color, None)?;
                let shadow_brush_data = shadow_brush.to_shader_data();
                this.render_text_glyphs_inner(glyphs_to_draw, &shadow_brush_data, shadow_t)
            })?;
        }

        self.render_text_glyphs_inner(glyphs_to_draw, brush_data, local_transform)
    }

    unsafe fn render_text_glyphs_inner(
        &self,
        glyphs_to_draw: &mut Vec<(glow::Texture, f32, f32, f32, f32)>,
        brush_data: &crate::handle::GlGradientData,
        local_transform: Option<&Transform2D>,
    ) -> Result<(), GlError> {
        if glyphs_to_draw.is_empty() {
            return Ok(());
        }

        glyphs_to_draw.sort_by_key(|&(tex, ..)| tex);

        let gl = &self.gl_context;
        self.text_program.use_program(gl);

        let tex = self.create_gradient_texture(brush_data).unwrap_or(None);

        self.text_program
            .bind_transform(gl, self.get_final_transform(local_transform));

        if let Some(t_id) = tex {
            self.text_program.bind_brush_data(gl, brush_data, 1);
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(t_id));
        } else {
            self.text_program.bind_brush_data(gl, brush_data, 0);
        }

        self.text_program.bind_texture(gl, 0);
        gl.active_texture(glow::TEXTURE0);

        gl.bind_vertex_array(Some(self.text_vao));

        let mut batch_vertices = Vec::new();

        for chunk in glyphs_to_draw.chunk_by(|a, b| a.0 == b.0) {
            let (tex, _, _, _, _) = chunk[0];

            batch_vertices.clear();
            batch_vertices.reserve(chunk.len() * 24);

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

        Ok(())
    }
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
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA32F as i32,
            num_pixels as i32,
            1,
            0,
            glow::RGBA,
            glow::FLOAT,
            PixelUnpackData::Slice(Some(slice::from_raw_parts(
                pixels.as_ptr() as *const u8,
                pixels.len() * size_of::<f32>(),
            ))),
        );
        Ok(Some(tex))
    }

    #[inline]
    fn get_final_transform(&self, local_transform: Option<&Transform2D>) -> Transform2D {
        let (sx, sy) = self.dpi_scale;
        let current = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(Transform2D::IDENTITY);
        let transform = if let Some(t) = local_transform {
            current.multiply(t)
        } else {
            current
        };
        transform.then_scale(sx, sy).multiply(&self.proj_transform)
    }

    #[inline]
    unsafe fn render_tessellated_geometry(
        &mut self,
        geometry: &lyon_tessellation::VertexBuffers<Vertex, u32>,
        brush_data: &crate::handle::GlGradientData,
        local_transform: Option<&Transform2D>,
        shadow: Option<&Shadow>,
        clip_path: Option<&Path>,
        bounds: (f32, f32, f32, f32),
    ) -> Result<(), GlError> {
        if geometry.vertices.is_empty() || geometry.indices.is_empty() {
            return Ok(());
        }

        if let Some(shadow) = shadow {
            self.render_shadow(
                shadow,
                bounds,
                clip_path,
                local_transform,
                |this, shadow_t| {
                    let shadow_brush = this.create_solid_color_brush(shadow.color, None)?;
                    let shadow_brush_data = shadow_brush.to_shader_data();
                    this.render_tessellated_geometry_inner(geometry, &shadow_brush_data, shadow_t)
                },
            )?;
        }

        self.render_tessellated_geometry_inner(geometry, brush_data, local_transform)
    }

    #[inline]
    unsafe fn render_tessellated_geometry_inner(
        &self,
        geometry: &lyon_tessellation::VertexBuffers<Vertex, u32>,
        brush_data: &crate::handle::GlGradientData,
        local_transform: Option<&Transform2D>,
    ) -> Result<(), GlError> {
        let gl = &self.gl_context;
        self.color_program.use_program(gl);
        self.color_program
            .bind_transform(gl, self.get_final_transform(local_transform));

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

        let vertex_bytes = slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * size_of::<V>(),
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
        // Here dpi_x and dpi_y could contain the extra scaling factor from local_transform
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
        &mut self,
        draw_x: f32,
        draw_y: f32,
        draw_w: f32,
        draw_h: f32,
        texture: NativeTexture,
        opacity: Option<f32>,
        tint: Option<Color>,
        local_transform: Option<&Transform2D>,
        shadow: Option<&Shadow>,
    ) -> Result<(), GlError> {
        if let Some(shadow) = shadow {
            unsafe {
                self.render_shadow(
                    shadow,
                    (draw_x, draw_y, draw_w, draw_h),
                    None,
                    local_transform,
                    |this, shadow_t| {
                        this.draw_texture_inner(
                            draw_x,
                            draw_y,
                            draw_w,
                            draw_h,
                            texture,
                            opacity,
                            Some(shadow.color),
                            shadow_t,
                        );
                        Ok(())
                    },
                )?;
            }
        }
        self.draw_texture_inner(
            draw_x,
            draw_y,
            draw_w,
            draw_h,
            texture,
            opacity,
            tint,
            local_transform,
        );
        Ok(())
    }

    #[inline]
    fn draw_texture_inner(
        &self,
        draw_x: f32,
        draw_y: f32,
        draw_w: f32,
        draw_h: f32,
        texture: NativeTexture,
        opacity: Option<f32>,
        tint: Option<Color>,
        local_transform: Option<&Transform2D>,
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
            self.texture_program
                .bind_transform(gl, self.get_final_transform(local_transform));
            self.texture_program.bind_texture(gl, 0);
            self.texture_program.bind_opacity(gl, opacity);
            self.texture_program.bind_tint(gl, tint);

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
