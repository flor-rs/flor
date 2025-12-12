#![allow(irrefutable_let_patterns)]
#![allow(unreachable_patterns)]
#![allow(unreachable_code)]
#![allow(clippy::needless_return)]

use crate::error::Error;
use crate::render::backend_error::FlorRenderError;
use crate::render::image_handle::FlorImageHandle;
use crate::render::surface_id::FlorSurfaceId;
#[cfg(feature = "svg")]
use crate::render::svg_handle::FlorSvgHandle;
use crate::render::text_format_handle::FlorTextFormatHandle;
use crate::render::FlorBrushHandle;
#[cfg(feature = "svg")]
use flor_graphics_base::SvgDrawOptions;
use flor_graphics_base::{
    Color, Gradient, ImageDrawOptions, Path, PathDrawOptions, Render, RenderContext,
    TextDrawOptions, Transform2D,
};
use graphics::D2DRender;
use platform::WindowId;

#[derive(Debug)]
pub enum FlorRender {
    #[cfg(feature = "direct2d")]
    GPU(D2DRender),
    // CPU(#[cfg(feature = "gdi")] GDIRender),
}

impl FlorRender {
    pub fn create(
        window_id: WindowId,
        width: u32,
        height: u32,
        wait_v_sync: bool,
    ) -> Result<Self, Error> {
        #[cfg(feature = "direct2d")]
        match D2DRender::create(window_id, width, height, wait_v_sync) {
            Ok(render) => return Ok(Self::GPU(render)),
            Err(err) => {
                log::error!("{}", err);
            }
        }

        // #[cfg(feature = "direct2d")]

        Err(Error::InitError("初始化渲染器失败".to_string()))
    }
}

impl RenderContext for FlorRender {
    type Error = FlorRenderError;
    type ImageHandle = FlorImageHandle;
    type SurfaceId = FlorSurfaceId;
    type BrushHandle = FlorBrushHandle;
    #[cfg(feature = "svg")]
    type SvgHandle = FlorSvgHandle;
    type TextFormatHandle = FlorTextFormatHandle;

    fn begin(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => g.begin()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c.begin()?,
        };
        Ok(())
    }

    fn end(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => g.end()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c.end()?,
        };
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => g.clear(color)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c.clear(color)?,
        };
        Ok(())
    }

    fn test(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => g.test()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c.test()?,
        };
        Ok(())
    }

    fn update_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => g.update_window_size(width, height)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c.update_window_size(width, height)?,
        };
        Ok(())
    }

    fn create_surface(&mut self, width: u32, height: u32) -> Result<Self::SurfaceId, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => Ok(FlorSurfaceId::D2DSurfaceId(
                g.create_surface(width, height)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => Ok(c.create_surface(width, height)?),
        }
    }

    fn set_render_target(&mut self, surface_id: &Self::SurfaceId) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                if let FlorSurfaceId::D2DSurfaceId(surface_id) = surface_id {
                    g.set_render_target(surface_id)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c.set_render_target(target)?,
        };
        Err(FlorRenderError::RenderNotFound)
    }

    fn reset_render_target(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => g.reset_render_target()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c.reset_render_target()?,
        };
        Ok(())
    }

    fn create_image_from_bytes(&mut self, bytes: &[u8]) -> Result<Self::ImageHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => Ok(FlorImageHandle::D2DImageHandle(
                g.create_image_from_bytes(bytes)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => Ok(c.create_image_from_bytes(bytes)?),
        }
    }

    fn create_image_from_raw_bytes(&mut self, raw_bytes: Vec<Vec<u8>>, width: u32, height: u32, delays: Vec<u16>) -> Result<Self::ImageHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => Ok(FlorImageHandle::D2DImageHandle(
                g.create_image_from_raw_bytes(raw_bytes,width,height,delays)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => Ok(c.create_image_from_bytes(bytes)?),
        }
    }

    #[cfg(feature = "svg")]
    fn create_svg(&mut self, bytes: &[u8]) -> Result<Self::SvgHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => Ok(FlorSvgHandle::D2DSvgHandle(g.create_svg(bytes)?)),
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => Ok(c.create_svg(bytes)?),
        }
    }

    fn create_text_format(
        &mut self,
        font_family_name: &str,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => Ok(FlorTextFormatHandle::D2DTextFormatHandle(
                g.create_text_format(font_family_name)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => Ok(c.create_text_format(font_family_name)?),
        }
    }

    fn measure_text(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
    ) -> Result<(f32, f32), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                // 解包 GPU 具体的 Format Handle
                if let FlorTextFormatHandle::D2DTextFormatHandle(inner_fmt) = text_format {
                    return Ok(g.measure_text(text, inner_fmt, width, height)?);
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                // 解包 CPU 具体的 Format Handle
                if let FlorTextFormatHandle::CPUTextFormatHandle(inner_fmt) = text_format {
                    let result = c.measure_text(text, inner_fmt, width, height)?;
                    return Ok(result);
                }
            }
        }
        // 如果后端匹配但 Handle 类型不对，或者没有启用的后端分支
        Err(FlorRenderError::RenderNotFound)
    }

    fn create_solid_color_brush(
        &mut self,
        color: Color,
        opacity: Option<f32>,
    ) -> Result<Self::BrushHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => Ok(FlorBrushHandle::D2DBrushHandle(
                g.create_solid_color_brush(color, opacity)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => Ok(c.create_solid_color_brush(color, opacity)?),
        }
    }

    fn create_gradient_brush(
        &mut self,
        gradient: Gradient,
    ) -> Result<Self::BrushHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => Ok(FlorBrushHandle::D2DBrushHandle(
                g.create_gradient_brush(gradient)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => Ok(c.create_gradient_brush(gradient)?),
        }
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
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                if let FlorImageHandle::D2DImageHandle(inner) = handle {
                    g.draw_image(inner, x, y, width, height, options)?;
                    return Ok(()); // 成功则直接返回
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                if let FlorImageHandle::CPUImageHandle(inner) = handle {
                    c.draw_image(inner, x, y, width, height, options)?;
                    return Ok(()); // 成功则直接返回
                }
            }
        };
        // 走到这里说明：虽然匹配到了 Backend，但是 Handle 类型对不上
        Err(FlorRenderError::RenderNotFound)
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
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                if let FlorSvgHandle::D2DSvgHandle(inner) = handle {
                    g.draw_svg(inner, x, y, width, height, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                if let FlorSvgHandle::CPUSvgHandle(inner) = handle {
                    c.draw_svg(inner, x, y, width, height, options)?;
                    return Ok(());
                }
            }
        };
        Err(FlorRenderError::RenderNotFound)
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
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                // 双重解包
                if let (
                    FlorTextFormatHandle::D2DTextFormatHandle(inner_fmt),
                    FlorBrushHandle::D2DBrushHandle(inner_brush),
                ) = (text_format, brush)
                {
                    g.draw_text(
                        text,
                        inner_fmt,
                        left,
                        top,
                        width,
                        height,
                        inner_brush,
                        options,
                    )?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                if let (
                    FlorTextFormatHandle::CPUTextFormatHandle(inner_fmt),
                    FlorBrushHandle::CPUBrushHandle(inner_brush),
                ) = (text_format, brush)
                {
                    c.draw_text(
                        text,
                        inner_fmt,
                        left,
                        top,
                        width,
                        height,
                        inner_brush,
                        options,
                    )?;
                    return Ok(());
                }
            }
        };
        Err(FlorRenderError::RenderNotFound)
    }

    fn draw_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        stroke_width: f32,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                if let FlorBrushHandle::D2DBrushHandle(inner) = brush {
                    g.draw_path(path, inner, stroke_width, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                if let FlorBrushHandle::CPUBrushHandle(inner) = brush {
                    c.draw_path(path, inner, stroke_width, options)?;
                    return Ok(());
                }
            }
        };
        Err(FlorRenderError::RenderNotFound)
    }

    fn fill_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                if let FlorBrushHandle::D2DBrushHandle(inner) = brush {
                    g.fill_path(path, inner, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                if let FlorBrushHandle::CPUBrushHandle(inner) = brush {
                    c.fill_path(path, inner, options)?;
                    return Ok(());
                }
            }
        };
        Err(FlorRenderError::RenderNotFound)
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
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                if let FlorBrushHandle::D2DBrushHandle(inner) = brush {
                    g.draw_quad(left, top, width, height, border_width, inner, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                if let FlorBrushHandle::CPUBrushHandle(inner) = brush {
                    c.draw_quad(left, top, width, height, border_width, inner, options)?;
                    return Ok(());
                }
            }
        };
        Err(FlorRenderError::RenderNotFound)
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
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                if let FlorBrushHandle::D2DBrushHandle(inner) = brush {
                    g.fill_quad(left, top, width, height, inner, corner_radius, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                if let FlorBrushHandle::CPUBrushHandle(inner) = brush {
                    c.fill_quad(left, top, width, height, inner, corner_radius, options)?;
                    return Ok(());
                }
            }
        }
        Err(FlorRenderError::RenderNotFound)
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
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => g.blur_quad(
                left,
                top,
                width,
                height,
                corner_radius,
                blur_radius,
                transform,
            )?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c.blur_region(left, top, width, height, radius, transform)?,
        };
        Ok(())
    }

    fn blur_path(
        &mut self,
        path: &Path,
        blur_radius: f32,
        transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => g.blur_path(path, blur_radius, transform)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => c..blur_path(path, blur_radius, transform)?,
        };
        Ok(())
    }

    fn set_clip(
        &mut self,
        surface_id: Option<&Self::SurfaceId>,
        rect: Option<(f32, f32, f32, f32)>,
    ) -> Result<(), Self::Error> {
        return match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRender::GPU(g) => {
                let d2d_surface_id = match surface_id {
                    Some(FlorSurfaceId::D2DSurfaceId(inner)) => Some(inner),
                    None => None,
                    _ => return Err(FlorRenderError::RenderNotFound),
                };
                g.set_clip(d2d_surface_id, rect)?;
                Ok(())
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRender::CPU(c) => {
                let cpu_surface_id = match surface_id {
                    Some(FlorSurfaceId::CPUSurfaceId(inner)) => Some(inner),
                    None => None,
                    _ => return Err(FlorRenderError::RenderNotFound),
                };

                c.set_clip(cpu_surface_id, rect)?;
                Ok(())
            }
        };
    }
}
