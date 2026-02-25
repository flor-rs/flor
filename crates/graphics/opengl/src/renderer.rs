use crate::error::GlError;
use crate::handle::GlImageHandle;
use crate::handle::GlSurfaceId;
use crate::handle::{GlBrushHandle, GlTextFormatHandle};
#[cfg(feature = "svg")]
use {crate::handle::GlSvgHandle, flor_base::graphics::SvgDrawOptions};

use crate::platform;
use flor_base::graphics::{
    Gradient, HitTestResult, ImageDrawOptions, Path, PathDrawOptions, Render, RenderContext,
    TextDrawOptions,
};
use flor_base::types::{Color, Transform2D};
use glow::HasContext;
use windows::Win32::Foundation::HWND;

mod config;
pub use config::*;

#[derive(Debug)]
pub struct GlRenderer {
    gl_context: glow::Context,
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
        let gl_context = platform::get_gl_context(hwnd.into(), config)?;
        unsafe {
            platform::set_vsync(wait_v_sync);
            gl_context.viewport(0, 0, width as i32, height as i32);
        }
        Ok(GlRenderer { gl_context })
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
        Ok(())
    }

    fn end(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        Ok(())
    }

    fn test(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_scale_factor(&mut self, dpi_x: f32, dpi_y: f32) -> Result<(), Self::Error> {
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
        Ok(GlImageHandle {})
    }

    fn create_image_from_raw_bytes(
        &mut self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<Self::ImageHandle, Self::Error> {
        Ok(GlImageHandle {})
    }
    #[cfg(feature = "svg")]
    fn create_svg(&mut self, bytes: &[u8]) -> Result<Self::SvgHandle, Self::Error> {
        todo!()
    }

    fn create_text_format(
        &mut self,
        font_family_name: &str,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        Ok(GlTextFormatHandle {})
    }
    #[cfg(feature = "memory-font")]
    fn create_text_format_from_bytes(
        &mut self,
        font_data: &[u8],
        ttc_index: u32,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        Ok(GlTextFormatHandle {})
    }

    fn measure_text(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
    ) -> Result<(f32, f32), Self::Error> {
        Ok((0., 0.))
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
        Ok(HitTestResult {
            text_index: 0,
            is_trailing: false,
            is_inside: false,
            is_trimmed: false,
            rect: (0.0, 0.0, 0.0, 0.0),
        })
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
        Ok((0., 0.))
    }

    fn create_solid_color_brush(
        &mut self,
        color: Color,
        opacity: Option<f32>,
    ) -> Result<Self::BrushHandle, Self::Error> {
        Ok(GlBrushHandle {})
    }

    fn create_gradient_brush(
        &mut self,
        gradient: &Gradient,
    ) -> Result<Self::BrushHandle, Self::Error> {
        Ok(GlBrushHandle {})
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
        todo!()
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
        Ok(())
    }

    fn draw_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        stroke_width: f32,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn fill_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
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
        Ok(())
    }

    fn blur_path(
        &mut self,
        path: &Path,
        blur_radius: f32,
        transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn push_clip(&mut self, rect: (f32, f32, f32, f32)) -> Result<(), Self::Error> {
        Ok(())
    }

    fn push_rounded_clip(
        &mut self,
        rect: (f32, f32, f32, f32),
        radius: f32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn push_path_clip(&mut self, path: &Path) -> Result<(), Self::Error> {
        Ok(())
    }

    fn pop_clip(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_clip_depth(&mut self) -> Result<u32, Self::Error> {
        Ok(0)
    }

    fn suspend_clip(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn resume_clip(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn push_transform(&mut self, transform: &Transform2D) -> Result<(), Self::Error> {
        Ok(())
    }

    fn pop_transform(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_transform_depth(&mut self) -> Result<u32, Self::Error> {
        Ok(0)
    }

    fn capture_snapshot(
        &mut self,
        rect: Option<(f32, f32, u32, u32)>,
    ) -> Result<Vec<u8>, Self::Error> {
        Ok(vec![])
    }
}
