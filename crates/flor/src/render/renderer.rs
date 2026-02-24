#![allow(irrefutable_let_patterns)]
#![allow(unreachable_patterns)]
#![allow(unreachable_code)]
#![allow(clippy::needless_return)]

use crate::error::Error;
use crate::render::image_handle::FlorImageHandle;
use crate::render::surface_id::FlorSurfaceId;
#[cfg(feature = "svg")]
use crate::render::svg_handle::FlorSvgHandle;
use crate::render::text_format_handle::FlorTextFormatHandle;
use crate::render::{FlorBrushHandle, FlorRendererError};
#[cfg(feature = "svg")]
use flor_base::graphics::SvgDrawOptions;
use flor_base::graphics::{
    Gradient, HitTestResult, ImageDrawOptions, Path, PathDrawOptions, Render, RenderContext,
    TextDrawOptions,
};
use flor_base::types::{Color, Transform2D};
#[cfg(feature = "direct2d")]
use graphics::D2DRenderer;
#[cfg(feature = "opengl")]
use graphics::GlRenderer;
use platform::WindowId;

#[derive(Debug)]
pub enum FlorRenderer {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DRenderer,
        #[cfg(feature = "opengl")] GlRenderer,
    ),
    // CPU(#[cfg(feature = "gdi")] GDIRender),
}

impl FlorRenderer {
    pub fn create(
        window_id: WindowId,
        width: u32,
        height: u32,
        wait_v_sync: bool,
    ) -> Result<Self, Error> {
        #[cfg(feature = "direct2d")]
        match D2DRenderer::create(window_id, width, height, wait_v_sync) {
            Ok(render) => return Ok(Self::GPU(render)),
            Err(err) => {
                log::error!("{}", err);
            }
        }

        #[cfg(feature = "opengl")]
        match GlRenderer::create(window_id, width, height, wait_v_sync) {
            Ok(render) => return Ok(Self::GPU(render)),
            Err(err) => {
                log::error!("{}", err);
            }
        }

        Err(Error::InitError("初始化渲染器失败".to_string()))
    }
}

impl RenderContext for FlorRenderer {
    type Error = FlorRendererError;
    type ImageHandle = FlorImageHandle;
    type SurfaceId = FlorSurfaceId;
    type BrushHandle = FlorBrushHandle;
    #[cfg(feature = "svg")]
    type SvgHandle = FlorSvgHandle;
    type TextFormatHandle = FlorTextFormatHandle;

    fn begin(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.begin()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.begin()?,
        };
        Ok(())
    }

    fn end(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.end()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.end()?,
        };
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.clear(color)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.clear(color)?,
        };
        Ok(())
    }

    fn test(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.test()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.test()?,
        };
        Ok(())
    }

    fn update_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.update_window_size(width, height)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.update_window_size(width, height)?,
        };
        Ok(())
    }

    fn set_scale_factor(&mut self, dpi_x: f32, dpi_y: f32) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.set_scale_factor(dpi_x, dpi_y)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.set_scale_factor(dpi_x, dpi_y)?,
        };
        Ok(())
    }

    fn create_surface(&mut self, width: u32, height: u32) -> Result<Self::SurfaceId, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(FlorSurfaceId::GPU(g.create_surface(width, height)?)),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.create_surface(width, height)?),
        }
    }

    fn set_render_target(&mut self, surface_id: &Self::SurfaceId) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => {
                if let FlorSurfaceId::GPU(surface_id) = surface_id {
                    g.set_render_target(surface_id)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.set_render_target(target)?,
        };
        Err(FlorRendererError::RenderNotFound)
    }

    fn reset_render_target(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.reset_render_target()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.reset_render_target()?,
        };
        Ok(())
    }

    fn create_image_from_bytes(&mut self, bytes: &[u8]) -> Result<Self::ImageHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(FlorImageHandle::GPU(g.create_image_from_bytes(bytes)?)),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.create_image_from_bytes(bytes)?),
        }
    }

    fn create_image_from_raw_bytes(
        &mut self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<Self::ImageHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(FlorImageHandle::GPU(
                g.create_image_from_raw_bytes(raw_bytes, width, height, delays)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.create_image_from_bytes(bytes)?),
        }
    }

    #[cfg(feature = "svg")]
    fn create_svg(&mut self, bytes: &[u8]) -> Result<Self::SvgHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(FlorSvgHandle::GPU(g.create_svg(bytes)?)),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.create_svg(bytes)?),
        }
    }

    fn create_text_format(
        &mut self,
        font_family_name: &str,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(FlorTextFormatHandle::GPU(
                g.create_text_format(font_family_name)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.create_text_format(font_family_name)?),
        }
    }

    #[cfg(feature = "memory-font")]
    fn create_text_format_from_bytes(
        &mut self,
        font_data: &[u8],
        ttc_index: u32,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(FlorTextFormatHandle::GPU(
                g.create_text_format_from_bytes(font_data, ttc_index)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.create_text_format_from_bytes(font_data, ttc_index)?),
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
            FlorRenderer::GPU(g) => {
                // 解包 GPU 具体的 Format Handle
                if let FlorTextFormatHandle::GPU(inner_fmt) = text_format {
                    return Ok(g.measure_text(text, inner_fmt, width, height)?);
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                // 解包 CPU 具体的 Format Handle
                if let FlorTextFormatHandle::CPUTextFormatHandle(inner_fmt) = text_format {
                    let result = c.measure_text(text, inner_fmt, width, height)?;
                    return Ok(result);
                }
            }
        }
        // 如果后端匹配但 Handle 类型不对，或者没有启用的后端分支
        Err(FlorRendererError::RenderNotFound)
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
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => {
                if let FlorTextFormatHandle::GPU(inner_fmt) = text_format {
                    return Ok(g.hit_test_point(text, inner_fmt, width, height, x, y)?);
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                if let FlorTextFormatHandle::D2DTextFormatHandle(inner_fmt) = text_format {
                    return Ok(g.hit_test_text_position(text, inner_fmt, width, height, x, y)?);
                }
            }
        }

        Err(FlorRendererError::RenderNotFound)
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
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => {
                if let FlorTextFormatHandle::GPU(inner_fmt) = text_format {
                    return Ok(g.hit_test_text_position(
                        text, inner_fmt, width, height, text_index, trailing,
                    )?);
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                if let FlorTextFormatHandle::D2DTextFormatHandle(inner_fmt) = text_format {
                    return Ok(g.hit_test_text_position(
                        text, inner_fmt, width, height, text_index, trailing,
                    )?);
                }
            }
        }

        Err(FlorRendererError::RenderNotFound)
    }

    fn create_solid_color_brush(
        &mut self,
        color: Color,
        opacity: Option<f32>,
    ) -> Result<Self::BrushHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(FlorBrushHandle::GPU(
                g.create_solid_color_brush(color, opacity)?,
            )),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.create_solid_color_brush(color, opacity)?),
        }
    }

    fn create_gradient_brush(
        &mut self,
        gradient: &Gradient,
    ) -> Result<Self::BrushHandle, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(FlorBrushHandle::GPU(g.create_gradient_brush(gradient)?)),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.create_gradient_brush(gradient)?),
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
            FlorRenderer::GPU(g) => {
                if let FlorImageHandle::GPU(inner) = handle {
                    g.draw_image(inner, x, y, width, height, options)?;
                    return Ok(()); // 成功则直接返回
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                if let FlorImageHandle::CPUImageHandle(inner) = handle {
                    c.draw_image(inner, x, y, width, height, options)?;
                    return Ok(()); // 成功则直接返回
                }
            }
        };
        // 走到这里说明：虽然匹配到了 Backend，但是 Handle 类型对不上
        Err(FlorRendererError::RenderNotFound)
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
            FlorRenderer::GPU(g) => {
                if let FlorSvgHandle::GPU(inner) = handle {
                    g.draw_svg(inner, x, y, width, height, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                if let SvgHandle::CPUSvgHandle(inner) = handle {
                    c.draw_svg(inner, x, y, width, height, options)?;
                    return Ok(());
                }
            }
        };
        Err(FlorRendererError::RenderNotFound)
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
            FlorRenderer::GPU(g) => {
                // 双重解包
                if let (FlorTextFormatHandle::GPU(inner_fmt), FlorBrushHandle::GPU(inner_brush)) =
                    (text_format, brush)
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
            FlorRenderer::CPU(c) => {
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
        Err(FlorRendererError::RenderNotFound)
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
            FlorRenderer::GPU(g) => {
                if let FlorBrushHandle::GPU(inner) = brush {
                    g.draw_path(path, inner, stroke_width, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                if let FlorBrushHandle::CPUBrushHandle(inner) = brush {
                    c.draw_path(path, inner, stroke_width, options)?;
                    return Ok(());
                }
            }
        };
        Err(FlorRendererError::RenderNotFound)
    }

    fn fill_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => {
                if let FlorBrushHandle::GPU(inner) = brush {
                    g.fill_path(path, inner, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                if let FlorBrushHandle::CPUBrushHandle(inner) = brush {
                    c.fill_path(path, inner, options)?;
                    return Ok(());
                }
            }
        };
        Err(FlorRendererError::RenderNotFound)
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
            FlorRenderer::GPU(g) => {
                if let FlorBrushHandle::GPU(inner) = brush {
                    g.draw_quad(left, top, width, height, border_width, inner, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                if let FlorBrushHandle::CPUBrushHandle(inner) = brush {
                    c.draw_quad(left, top, width, height, border_width, inner, options)?;
                    return Ok(());
                }
            }
        };
        Err(FlorRendererError::RenderNotFound)
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
            FlorRenderer::GPU(g) => {
                if let FlorBrushHandle::GPU(inner) = brush {
                    g.fill_quad(left, top, width, height, inner, corner_radius, options)?;
                    return Ok(());
                }
            }
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => {
                if let FlorBrushHandle::CPUBrushHandle(inner) = brush {
                    c.fill_quad(left, top, width, height, inner, corner_radius, options)?;
                    return Ok(());
                }
            }
        }
        Err(FlorRendererError::RenderNotFound)
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
            FlorRenderer::GPU(g) => g.blur_quad(
                left,
                top,
                width,
                height,
                corner_radius,
                blur_radius,
                transform,
            )?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.blur_region(left, top, width, height, radius, transform)?,
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
            FlorRenderer::GPU(g) => g.blur_path(path, blur_radius, transform)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c..blur_path(path, blur_radius, transform)?,
        };
        Ok(())
    }

    fn push_clip(&mut self, rect: (f32, f32, f32, f32)) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.push_clip(rect)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.push_clip(rect)?,
        };
        Ok(())
    }

    fn push_rounded_clip(
        &mut self,
        rect: (f32, f32, f32, f32),
        radius: f32,
    ) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.push_rounded_clip(rect, radius)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.push_rounded_clip(rect, radius)?,
        };
        Ok(())
    }

    fn push_path_clip(&mut self, path: &Path) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.push_path_clip(path)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.push_path_clip(path)?,
        };
        Ok(())
    }

    fn pop_clip(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.pop_clip(target_depth)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.pop_clip(target_depth)?,
        };
        Ok(())
    }

    fn get_clip_depth(&mut self) -> Result<u32, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(g.get_clip_depth()?),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.get_clip_depth()?),
        }
    }

    fn suspend_clip(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.suspend_clip()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.suspend_clip()?,
        };
        Ok(())
    }

    fn resume_clip(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.resume_clip()?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.resume_clip()?,
        };
        Ok(())
    }

    fn push_transform(&mut self, transform: &Transform2D) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.push_transform(transform)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.push_transform(transform)?,
        };
        Ok(())
    }

    fn pop_transform(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.pop_transform(target_depth)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.pop_transform(target_depth)?,
        };
        Ok(())
    }

    fn get_transform_depth(&mut self) -> Result<u32, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => Ok(g.get_transform_depth()?),
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => Ok(c.get_transform_depth()?),
        }
    }

    fn capture_snapshot(
        &mut self,
        rect: Option<(f32, f32, u32, u32)>,
    ) -> Result<Vec<u8>, Self::Error> {
        let result = match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorRenderer::GPU(g) => g.capture_snapshot(rect)?,
            #[cfg(feature = "cpu-render-backend")]
            FlorRenderer::CPU(c) => c.capture_snapshot(rect)?,
        };
        Ok(result)
    }
}
