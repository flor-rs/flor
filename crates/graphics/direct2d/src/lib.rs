use crate::handle::D2DBrushHandle;
use crate::handle::D2DImageHandle;
use crate::handle::D2DSurfaceId;
use crate::handle::D2DTextFormatHandle;
#[cfg(feature = "svg")]
use crate::handle::{D2DSvgHandle, SvgShadowCache};
use crate::into_d2d_transform::IntoD2DTransform;
use crate::util::color::AsD2dColor;
use crate::util::encode::encode_unicode;
use flor_graphics_base::{
    Color, Error, Gradient, ImageDrawOptions, ParagraphAlignment, Path, PathCommand,
    PathDrawOptions, Render, RenderContext, ScaleMode, TextAlignment, TextDrawOptions,
    TextFormatHandle, TextTrimming, Transform2D, WordWrapping,
};
use log::debug;
use lru::LruCache;

#[cfg(feature = "svg")]
use crate::base::SvgDrawOptions;
#[cfg(feature = "svg")]
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem::ManuallyDrop;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::OnceLock;
use std::{fmt, slice};
use windows::core::{w, Interface, HSTRING, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_BEZIER_SEGMENT, D2D1_BORDER_MODE, D2D1_BORDER_MODE_HARD,
    D2D1_COLOR_F, D2D1_COMPOSITE_MODE_SOURCE_OVER, D2D1_FIGURE_BEGIN_FILLED,
    D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN, D2D1_GRADIENT_STOP, D2D1_PIXEL_FORMAT,
    D2D_POINT_2U, D2D_RECT_F, D2D_RECT_U, D2D_SIZE_F, D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    CLSID_D2D1Crop, CLSID_D2D1GaussianBlur, CLSID_D2D1Shadow, D2D1CreateFactory, ID2D1Bitmap1,
    ID2D1Brush, ID2D1CommandList, ID2D1DeviceContext, ID2D1DeviceContext5, ID2D1Effect,
    ID2D1Factory2, ID2D1Geometry, ID2D1HwndRenderTarget, ID2D1Image, ID2D1ImageBrush, ID2D1Layer,
    ID2D1PathGeometry1, ID2D1RenderTarget, ID2D1SolidColorBrush, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
    D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_PROPERTIES1, D2D1_BRUSH_PROPERTIES,
    D2D1_BUFFER_PRECISION_8BPC_UNORM, D2D1_COLOR_INTERPOLATION_MODE_PREMULTIPLIED,
    D2D1_COLOR_SPACE_SRGB, D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_NONE, D2D1_CROP_PROP_RECT,
    D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_EXTEND_MODE_CLAMP, D2D1_FACTORY_TYPE_MULTI_THREADED,
    D2D1_FEATURE_LEVEL_DEFAULT, D2D1_GAUSSIANBLUR_PROP_BORDER_MODE,
    D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_IMAGE_BRUSH_PROPERTIES, D2D1_INTERPOLATION_MODE_LINEAR, D2D1_LAYER_OPTIONS1_NONE,
    D2D1_LAYER_PARAMETERS1, D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
    D2D1_MAP_OPTIONS_READ, D2D1_PRESENT_OPTIONS_IMMEDIATELY, D2D1_PRESENT_OPTIONS_NONE,
    D2D1_PROPERTY_TYPE_ENUM, D2D1_PROPERTY_TYPE_FLOAT, D2D1_PROPERTY_TYPE_VECTOR4,
    D2D1_QUADRATIC_BEZIER_SEGMENT, D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
    D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
    D2D1_ROUNDED_RECT, D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION, D2D1_SHADOW_PROP_COLOR,
    D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL,
    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_LINE_SPACING_METHOD_DEFAULT,
    DWRITE_LINE_SPACING_METHOD_UNIFORM, DWRITE_MEASURING_MODE_NATURAL,
    DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_PARAGRAPH_ALIGNMENT_FAR,
    DWRITE_PARAGRAPH_ALIGNMENT_NEAR, DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
    DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_TEXT_METRICS,
    DWRITE_TRIMMING, DWRITE_TRIMMING_GRANULARITY_CHARACTER, DWRITE_TRIMMING_GRANULARITY_NONE,
    DWRITE_TRIMMING_GRANULARITY_WORD, DWRITE_WORD_WRAPPING_CHARACTER, DWRITE_WORD_WRAPPING_NO_WRAP,
    DWRITE_WORD_WRAPPING_WRAP,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Imaging::{CLSID_WICImagingFactory, IWICImagingFactory};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
#[cfg(feature = "svg")]
use windows::{
    Win32::Foundation::HGLOBAL,
    Win32::Graphics::Direct2D::{
        ID2D1SvgDocument, D2D1_SVG_ATTRIBUTE_POD_TYPE_FLOAT, D2D1_SVG_ATTRIBUTE_POD_TYPE_VIEWBOX,
        D2D1_SVG_VIEWBOX,
    },
    Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal,
    Win32::System::Com::STREAM_SEEK_SET,
};
use windows_numerics::{Matrix3x2, Vector2};

// 只能在 Windows 平台启用 direct2d 后端
#[cfg(not(target_os = "windows"))]
compile_error!("The 'direct2d' feature is only supported on Windows platforms.");

mod brush;
mod error;

mod util;
pub mod base {
    pub use flor_graphics_base::*;
}
pub mod into_d2d_transform;

pub mod handle;

pub use {error::*, util::*};

pub static RENDER_FACTORY: OnceLock<RenderFactory> = OnceLock::new();

const D2D1_PIXEL_FORMAT: D2D1_PIXEL_FORMAT = D2D1_PIXEL_FORMAT {
    format: DXGI_FORMAT_B8G8R8A8_UNORM,
    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
};

pub struct RenderFactory {
    pub factory: ID2D1Factory2,

    pub dpi_x: f32,
    pub dpi_y: f32,

    pub d2d1_render_target_properties: D2D1_RENDER_TARGET_PROPERTIES,

    pub wic_factory: IWICImagingFactory,
    pub write_factory: IDWriteFactory,
    pub d2d1_bitmap_properties1: D2D1_BITMAP_PROPERTIES1,
}

unsafe impl Sync for RenderFactory {}
unsafe impl Send for RenderFactory {}

impl RenderFactory {
    pub fn try_init() -> Result<(), windows::core::Error> {
        debug!("init render factory.");
        if RENDER_FACTORY.get().is_none() {
            let render_factory = unsafe {
                let factory =
                    D2D1CreateFactory::<ID2D1Factory2>(D2D1_FACTORY_TYPE_MULTI_THREADED, None)?;
                let mut dpi_x = 0f32;
                let mut dpi_y = 0f32;
                factory.GetDesktopDpi(&mut dpi_x, &mut dpi_y);

                let d2d1_render_target_properties = D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                    pixelFormat: D2D1_PIXEL_FORMAT,
                    dpiX: dpi_x,
                    dpiY: dpi_y,
                    usage: D2D1_RENDER_TARGET_USAGE_NONE,
                    minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
                };

                let d2d1_bitmap_properties1 = D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT,
                    dpiX: dpi_x,
                    dpiY: dpi_y,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                    colorContext: ManuallyDrop::new(None),
                };

                Self {
                    factory,
                    dpi_x,
                    dpi_y,
                    d2d1_render_target_properties,
                    d2d1_bitmap_properties1,
                    wic_factory: CoCreateInstance(
                        &CLSID_WICImagingFactory,
                        None,
                        CLSCTX_INPROC_SERVER,
                    )?,
                    write_factory: DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?,
                }
            };
            let _ = RENDER_FACTORY.set(render_factory);
        }
        Ok(())
    }

    pub fn get<'a>() -> &'a RenderFactory {
        RENDER_FACTORY
            .get()
            .expect("RenderFactory not initialized.")
    }

    pub fn create_render(
        &self,
        hwnd: HWND,
        width: u32,
        height: u32,
        wait_v_sync: bool,
    ) -> Result<D2DRender, windows::core::Error> {
        unsafe {
            debug!("create window render.");
            let size = D2D_SIZE_U { width, height };

            let rf = RenderFactory::get();

            let render = self.factory.CreateHwndRenderTarget(
                &rf.d2d1_render_target_properties,
                &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                    hwnd,
                    pixelSize: size,
                    presentOptions: match wait_v_sync {
                        true => D2D1_PRESENT_OPTIONS_NONE,
                        false => D2D1_PRESENT_OPTIONS_IMMEDIATELY,
                    },
                },
            )?;
            render.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            render.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            let current_render = render.cast::<ID2D1DeviceContext>()?;
            let scratch_brush =
                current_render.CreateSolidColorBrush(&D2D1_COLOR_F::default(), None)?;
            Ok(D2DRender {
                size,
                hwnd_render: render,
                current_render,
                scratch_brush,
                cached_shadow_effect: None,
                cached_blur_effect: None,
                cached_crop_effect: None,
                cached_blur_bitmap: None,
                last_blur_radius: 0.0,
                lur: LruCache::new(NonZeroUsize::new(512).unwrap_or_else(|| unreachable!())),
                layer_pool: vec![],
            })
        }
    }
}

pub struct D2DRender {
    pub size: D2D_SIZE_U,
    pub hwnd_render: ID2D1HwndRenderTarget,
    pub current_render: ID2D1DeviceContext,
    // d2d1_render_target_properties: D2D1_RENDER_TARGET_PROPERTIES,
    pub scratch_brush: ID2D1SolidColorBrush,
    pub cached_shadow_effect: Option<ID2D1Effect>,
    pub cached_blur_effect: Option<ID2D1Effect>, // 缓存 ID2D1Effect 对象
    pub cached_crop_effect: Option<ID2D1Effect>, // 缓存 Crop, 避免模糊时采样到脏数据
    pub cached_blur_bitmap: Option<ID2D1Bitmap1>, // 缓存用于截屏的 Bitmap (Scratch Bitmap)
    pub last_blur_radius: f32,                   // 缓存上次使用的半径
    pub lur: LruCache<u64, ID2D1CommandList>,
    pub layer_pool: Vec<ID2D1Layer>,
}
impl Debug for D2DRender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 创建一个 debug 结构体构建器
        let mut debug_struct = f.debug_struct("D2DRender");

        // 1. 打印尺寸信息
        debug_struct.field("size", &self.size);

        // 2. 打印 COM 对象的指针地址
        // 这对于调试资源是否被正确创建或是否发生了变化非常有用
        debug_struct.field(
            "hwnd_render_ptr",
            &format!("{:p}", self.hwnd_render.as_raw()),
        );
        debug_struct.field(
            "current_render_ptr",
            &format!("{:p}", self.current_render.as_raw()),
        );

        // 3. 打印辅助资源的状态
        debug_struct.field(
            "scratch_brush_ptr",
            &format!("{:p}", self.scratch_brush.as_raw()),
        );

        // 4. 打印缓存特效的状态
        // Option 类型我们可以手动处理一下显示
        match &self.cached_shadow_effect {
            Some(effect) => {
                debug_struct.field(
                    "cached_shadow_effect",
                    &format!("Some({:p})", effect.as_raw()),
                );
            }
            None => {
                debug_struct.field("cached_shadow_effect", &"None");
            }
        }

        // 完成格式化
        debug_struct.finish()
    }
}
impl Render for D2DRender {
    type HWND = HWND;
    type Render = Self;

    fn create(
        hwnd: impl Into<Self::HWND>,
        width: u32,
        height: u32,
        wait_v_sync: bool,
    ) -> Result<Self::Render, Self::Error> {
        debug!("create window render.");
        RenderFactory::try_init()?;
        Ok(RenderFactory::get().create_render(hwnd.into(), width, height, wait_v_sync)?)
    }
}

impl RenderContext for D2DRender {
    type Error = D2DBackendError;
    type ImageHandle = D2DImageHandle;
    type SurfaceId = D2DSurfaceId;
    type BrushHandle = D2DBrushHandle;
    #[cfg(feature = "svg")]
    type SvgHandle = D2DSvgHandle;
    type TextFormatHandle = D2DTextFormatHandle;

    fn begin(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.hwnd_render.BeginDraw();
        }
        Ok(())
    }

    fn end(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.hwnd_render.EndDraw(None, None)?;
        }
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<(), Self::Error> {
        unsafe {
            self.hwnd_render.Clear(Some(&color.as_d2d_color()));
        }
        Ok(())
    }

    fn test(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.hwnd_render.Clear(Some(&D2D1_COLOR_F {
                r: 1.0, // 修改为红色
                g: 1.0, // 修改为绿色
                b: 1.0, // 修改为蓝色
                a: 1.0, // 透明度 0.4
            }));
            let brush = self.hwnd_render.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.0, // 修改为红色
                    g: 1.0, // 修改为绿色
                    b: 0.0, // 修改为蓝色
                    a: 0.5,
                },
                None,
            )?;

            self.hwnd_render.DrawRectangle(
                &D2D_RECT_F {
                    left: 50.0,
                    top: 50.0,
                    right: 200.0,
                    bottom: 200.0,
                },
                &brush,
                5.0,
                None,
            );
            Ok(())
        }
    }

    fn update_window_size(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        self.size = D2D_SIZE_U { width, height };
        debug!("update_window_size {:?}", self.size);
        unsafe {
            self.hwnd_render.Resize(&self.size)?;
        }
        Ok(())
    }

    fn set_scale_factor(&mut self, dpi_x: f32, dpi_y: f32) -> Result<(), Self::Error> {
        unsafe {
            self.current_render.SetDpi(dpi_x, dpi_y);
            Ok(())
        }
    }

    fn create_surface(&mut self, width: u32, height: u32) -> Result<Self::SurfaceId, Self::Error> {
        let render = unsafe {
            self.hwnd_render.CreateCompatibleRenderTarget(
                Some(&D2D_SIZE_F {
                    width: width as f32,
                    height: height as f32,
                }),
                None,
                None,
                D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_NONE,
            )?
        };
        Ok(D2DSurfaceId::new(render))
    }

    fn set_render_target(&mut self, surface_id: &Self::SurfaceId) -> Result<(), Self::Error> {
        self.current_render = surface_id.raw().cast::<ID2D1DeviceContext>()?;
        self.cached_shadow_effect = None;
        self.cached_blur_effect = None;
        self.cached_crop_effect = None;
        self.cached_blur_bitmap = None;
        self.last_blur_radius = -1.0;
        Ok(())
    }

    fn reset_render_target(&mut self) -> Result<(), Self::Error> {
        self.current_render = self.hwnd_render.cast::<ID2D1DeviceContext>()?;
        self.cached_shadow_effect = None;
        self.cached_blur_effect = None;
        self.cached_crop_effect = None;
        self.cached_blur_bitmap = None;
        self.last_blur_radius = -1.0;
        Ok(())
    }

    // 在你的 impl 中：
    fn create_image_from_bytes(&mut self, bytes: &[u8]) -> Result<Self::ImageHandle, Self::Error> {
        Ok(D2DImageHandle::from_bytes(
            bytes,
            &self.current_render,
            RenderFactory::get(),
        )?)
    }

    fn create_image_from_raw_bytes(
        &mut self,
        raw_bytes: Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<Self::ImageHandle, Self::Error> {
        Ok(D2DImageHandle::from_raw_bytes(
            raw_bytes,
            width,
            height,
            delays,
            &self.current_render,
        )?)
    }

    #[cfg(feature = "svg")]
    fn create_svg(&mut self, bytes: &[u8]) -> Result<Self::SvgHandle, Self::Error> {
        unsafe {
            if bytes.is_empty() {
                return Err(
                    windows::core::Error::from(windows::Win32::Foundation::E_INVALIDARG).into(),
                );
            }

            // 1. 创建自动增长的 IStream (传 None 让它内部自动分配)
            // 参数2 (true): 释放 IStream 时自动释放内存
            let stream = CreateStreamOnHGlobal(HGLOBAL::default(), true)?;

            // 2. 写入数据
            // IStream::Write 接收 *c_void, 长度, 和实际写入量的指针
            let mut bytes_written: u32 = 0;
            stream
                .Write(
                    bytes.as_ptr() as *const c_void,
                    bytes.len() as u32,
                    Some(&mut bytes_written),
                )
                .ok()?;

            if bytes_written as usize != bytes.len() {
                return Err(windows::core::Error::from(windows::Win32::Foundation::E_FAIL).into());
            }

            // 3. 【关键步骤】将流指针重置回开头！
            // 如果不重置，CreateSvgDocument 会从流末尾读取，读到空，报错参数错误
            stream.Seek(0, STREAM_SEEK_SET, None)?;

            // 2. 获取 ID2D1DeviceContext5 接口
            // 只有 Context5 及以上才支持 SVG。
            // 使用 cast() 动态转换接口，如果系统版本太低不支持，这里会报错
            let dc5: ID2D1DeviceContext5 = self.current_render.cast()?;

            // 3. 创建 SVG Document
            // viewportSize 传默认值 (0,0)，让 D2D 自动解析 SVG 内部尺寸
            let viewport_size = D2D_SIZE_F {
                width: 50.,
                height: 50.0,
            };
            let svg_document = dc5.CreateSvgDocument(&stream, viewport_size)?;

            let intrinsic_size = get_svg_size(&svg_document);

            if let Ok((w, h)) = intrinsic_size {
                // 如果成功读到了 viewBox 或 width/height，
                // 必须把文档的 ViewportSize 设置成这个真实值！
                // 否则 DrawSvgDocument 时可能会按照 1x1 进行裁剪或缩放。
                let real_size = D2D_SIZE_F {
                    width: w,
                    height: h,
                };
                svg_document.SetViewportSize(real_size)?;
            }

            Ok(D2DSvgHandle::new(svg_document))
        }
    }

    fn create_text_format(
        &mut self,
        font_family_name: &str,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        let text_format = unsafe {
            let text_format = RenderFactory::get().write_factory.CreateTextFormat(
                PCWSTR::from_raw(encode_unicode(font_family_name).as_ptr()),
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                16.0,
                w!(""),
            )?;
            text_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING)?;
            text_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_NEAR)?;
            text_format
        };
        Ok(D2DTextFormatHandle::new(text_format, font_family_name))
    }

    fn measure_text(
        &self,
        text: &str,
        text_format: &Self::TextFormatHandle,
        width: f32,
        height: f32,
    ) -> Result<(f32, f32), Self::Error> {
        unsafe {
            let text_layout = RenderFactory::get().write_factory.CreateTextLayout(
                &encode_unicode(text),
                text_format.raw(),
                width,
                height,
            )?;
            let mut text_metrics = DWRITE_TEXT_METRICS {
                left: 0.0,
                top: 0.0,
                width: 0.0,
                widthIncludingTrailingWhitespace: 0.0,
                height: 0.0,
                layoutWidth: 0.0,
                layoutHeight: 0.0,
                maxBidiReorderingDepth: 0,
                lineCount: 0,
            };
            text_layout.GetMetrics(&mut text_metrics)?;
            Ok((text_metrics.width, text_metrics.height))
        }
    }

    fn create_solid_color_brush(
        &mut self,
        color: Color,
        opacity: Option<f32>,
    ) -> Result<Self::BrushHandle, Self::Error> {
        unsafe {
            let props = opacity.map(|op| D2D1_BRUSH_PROPERTIES {
                opacity: op,
                transform: Default::default(),
            });

            let props = props.as_ref().map(|p| p as *const D2D1_BRUSH_PROPERTIES);

            let brush = self
                .current_render
                .CreateSolidColorBrush(&color.as_d2d_color(), props)?;

            Ok(D2DBrushHandle::new(brush.cast::<ID2D1Brush>()?))
        }
    }

    fn create_gradient_brush(
        &mut self,
        gradient: Gradient,
    ) -> Result<Self::BrushHandle, Self::Error> {
        unsafe {
            let brush: ID2D1Brush = match gradient {
                Gradient::Linear { start, end, colors } => {
                    // 构建 D2D 渐变停靠点和颜色数组
                    let stops = colors
                        .iter()
                        .map(|(pos, color)| D2D1_GRADIENT_STOP {
                            position: *pos,
                            color: color.as_d2d_color(),
                        })
                        .collect::<Vec<D2D1_GRADIENT_STOP>>();

                    // 创建渐变停止集合
                    let stops_collection = self.current_render.CreateGradientStopCollection(
                        &stops,
                        D2D1_COLOR_SPACE_SRGB,
                        D2D1_COLOR_SPACE_SRGB,
                        D2D1_BUFFER_PRECISION_8BPC_UNORM,
                        D2D1_EXTEND_MODE_CLAMP,
                        D2D1_COLOR_INTERPOLATION_MODE_PREMULTIPLIED,
                    )?;

                    // 创建线性渐变 Brush
                    self.current_render
                        .CreateLinearGradientBrush(
                            &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                                startPoint: Vector2 {
                                    X: start.0,
                                    Y: start.1,
                                },
                                endPoint: Vector2 { X: end.0, Y: end.1 },
                            },
                            None,
                            &stops_collection,
                        )?
                        .cast()?
                }
                Gradient::Radial {
                    center,
                    radius,
                    colors,
                } => {
                    let stops: Vec<D2D1_GRADIENT_STOP> = colors
                        .iter()
                        .map(|(pos, color)| D2D1_GRADIENT_STOP {
                            position: *pos,
                            color: color.as_d2d_color(),
                        })
                        .collect();

                    let stops_collection = self.current_render.CreateGradientStopCollection(
                        &stops,
                        D2D1_COLOR_SPACE_SRGB,
                        D2D1_COLOR_SPACE_SRGB,
                        D2D1_BUFFER_PRECISION_8BPC_UNORM,
                        D2D1_EXTEND_MODE_CLAMP,
                        D2D1_COLOR_INTERPOLATION_MODE_PREMULTIPLIED,
                    )?;

                    self.current_render
                        .CreateRadialGradientBrush(
                            &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                                center: Vector2 {
                                    X: center.0,
                                    Y: center.1,
                                },
                                radiusX: radius,
                                radiusY: radius,
                                ..Default::default()
                            },
                            None,
                            &stops_collection,
                        )?
                        .cast()?
                }
            };

            Ok(D2DBrushHandle::new(brush))
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
        let frame_index = options.and_then(|v| v.frame_index).unwrap_or(0);
        // 1. 资源与参数解构
        let bitmap = match handle.bitmaps.get(frame_index) {
            None => {
                return Err(D2DBackendError::from(Error::ImageFrameNotFound(
                    frame_index,
                )));
            }
            Some(bitmap) => bitmap,
        };

        let (scale_mode, transform, global_opacity, shadow, _frame_index) = match options {
            None => (ScaleMode::None, None, 1.0, None, 0),
            Some(opt) => (
                opt.scale_mode.unwrap_or(ScaleMode::None),
                opt.transform,
                opt.opacity.unwrap_or(1.0),
                opt.shadow,
                opt.frame_index.unwrap_or(0),
            ),
        };

        unsafe {
            let size = bitmap.GetSize();
            let target_w = width.unwrap_or(size.width);
            let target_h = height.unwrap_or(size.height);

            // ---- 2. 布局计算 (Layout Calculation) ----
            // 计算图片为了适应目标框，实际应该缩放成多大，以及偏移多少
            let (mut final_w, mut final_h) = (size.width, size.height);
            let (mut offset_x, mut offset_y) = (0.0, 0.0);
            let mut need_clip = false;

            match scale_mode {
                ScaleMode::None => { /* 原图 */ }
                ScaleMode::Fit => {
                    let ratio = (target_w / size.width).min(target_h / size.height);
                    final_w = size.width * ratio;
                    final_h = size.height * ratio;
                    offset_x = (target_w - final_w) / 2.0;
                    offset_y = (target_h - final_h) / 2.0;
                }
                ScaleMode::Cover => {
                    let ratio = (target_w / size.width).max(target_h / size.height);
                    final_w = size.width * ratio;
                    final_h = size.height * ratio;
                    // 居中偏移
                    offset_x = (target_w - final_w) / 2.0;
                    offset_y = (target_h - final_h) / 2.0;
                    need_clip = true;
                }
                ScaleMode::Stretch => {
                    final_w = target_w;
                    final_h = target_h;
                }
                _ => {}
            }

            // ---- 3. 坐标系准备 ----
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            // A. 用户变换矩阵 (User Transform)
            // 这个矩阵决定了 "x, y, target_w, target_h" 这个框在世界中的位置/旋转
            let user_matrix = if let Some(t) = transform {
                t.into_transform()
            } else {
                Matrix3x2::identity()
            };

            // B. 布局矩阵 (Layout Matrix)
            // 这个矩阵决定了 "图片" 如何填入 "x, y, target_w, target_h" 这个框
            let scale_x = final_w / size.width;
            let scale_y = final_h / size.height;
            // 注意：这里加上 x, y 是因为 offset 是相对于 x, y 的
            let layout_matrix = Matrix3x2::scale(scale_x, scale_y)
                * Matrix3x2::translation(x + offset_x, y + offset_y);

            // C. 预先计算好的组合矩阵
            // pre_clip_transform: 此时坐标系原点在世界原点，但应用了用户的旋转位移。
            // 在这个坐标系下，裁剪框就是 (x, y, target_w, target_h)。
            let pre_clip_transform = user_matrix * old_transform;

            // final_transform: 图片绘制的最终矩阵
            let final_transform = layout_matrix * pre_clip_transform; // layout * user * world

            // ---- 4. 定义核心绘制逻辑 (闭包) ----
            // 这样我们可以把这段逻辑传给 with_content_clip 或者直接执行
            let draw_content = |render: &mut Self| -> Result<(), Self::Error> {
                // 应用最终矩阵：此时 (0,0) 对应图片左上角，(width, height) 对应图片右下角
                render.current_render.SetTransform(&final_transform);

                // 4.1 智能 Layer 判断 (半透明 + 阴影 必须用 Layer)
                let has_shadow = shadow.is_some();
                let is_transparent = global_opacity < 0.999;
                let use_transparency_layer = has_shadow && is_transparent;

                let mut active_layer: Option<ID2D1Layer> = None;

                if use_transparency_layer {
                    // === 优化：从池中获取 Layer ===
                    let layer = render.get_layer()?;

                    let layer_params = D2D1_LAYER_PARAMETERS1 {
                        contentBounds: D2D_RECT_F {
                            left: -f32::INFINITY,
                            top: -f32::INFINITY,
                            right: f32::INFINITY,
                            bottom: f32::INFINITY,
                        },
                        geometricMask: ManuallyDrop::new(None),
                        maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                        maskTransform: Matrix3x2::identity(),
                        opacity: global_opacity, // Layer 负责处理整体透明度
                        opacityBrush: ManuallyDrop::new(None),
                        layerOptions: D2D1_LAYER_OPTIONS1_NONE,
                    };
                    render.current_render.PushLayer(&layer_params, &layer);
                    active_layer = Some(layer);
                }

                // 4.2 绘制阴影
                if let Some(shadow_opts) = shadow {
                    // A. 锁住缓存
                    let mut cache = handle.frame_shadow_cache.lock();

                    // B. 尝试获取该帧对应的 Effect
                    // 如果 LRU 里有，直接拿出来用（D2D 内部缓存还在）
                    let shadow_effect = if let Some(effect) = cache.get(&frame_index) {
                        effect.clone() // Clone COM 指针是廉价的
                    } else {
                        // C. 缓存未命中（第一次播到这一帧，或者被 LRU 淘汰了）
                        // 创建新 Effect
                        let new_effect = render.current_render.CreateEffect(&CLSID_D2D1Shadow)?;

                        // 【永久绑定】：只要这个 Effect 活着，它就永远绑定这一帧的 Bitmap
                        new_effect.SetInput(0, bitmap, true);

                        // 存入 LRU (可能会挤掉很久没用的帧)
                        cache.put(frame_index, new_effect.clone());
                        new_effect
                    };

                    // D. 更新参数 (半径变了 D2D 会重算，没变则复用)
                    shadow_effect.SetValue(
                        D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                        D2D1_PROPERTY_TYPE_FLOAT,
                        &shadow_opts.blur_radius.to_ne_bytes(),
                    )?;

                    let d2d_color = shadow_opts.color.as_d2d_color();
                    shadow_effect.SetValue(
                        D2D1_SHADOW_PROP_COLOR.0 as u32,
                        D2D1_PROPERTY_TYPE_VECTOR4,
                        slice::from_raw_parts(&d2d_color as *const _ as *const u8, 16),
                    )?;

                    // E. 绘制
                    let offset = Vector2 {
                        X: shadow_opts.offset_x,
                        Y: shadow_opts.offset_y,
                    };

                    render.current_render.DrawImage(
                        &shadow_effect.cast::<ID2D1Image>()?,
                        Some(&offset),
                        None,
                        D2D1_INTERPOLATION_MODE_LINEAR,
                        D2D1_COMPOSITE_MODE_SOURCE_OVER,
                    );
                }

                // 4.3 绘制图片
                // 如果用了 Layer，Layer 负责透明度，这里画 1.0；否则直接画 global_opacity
                let draw_opacity = if use_transparency_layer {
                    1.0
                } else {
                    global_opacity
                };

                let local_rect = D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: size.width,
                    bottom: size.height,
                };

                render.current_render.DrawBitmap(
                    bitmap.deref(),
                    Some(&local_rect),
                    draw_opacity,
                    D2D1_INTERPOLATION_MODE_LINEAR,
                    None,
                    None,
                );

                // 4.4 恢复 Layer
                if let Some(layer) = active_layer {
                    render.current_render.PopLayer();
                    // === 优化：归还 Layer 到池中 ===
                    render.return_layer(layer);
                }

                Ok(())
            };

            // ---- 5. 执行绘制流程 (应用裁剪) ----

            if need_clip {
                // 定义裁剪区域 (在 User Space 下)
                let clip_rect = D2D_RECT_F {
                    left: x,
                    top: y,
                    right: x + target_w,
                    bottom: y + target_h,
                };

                // 先切换到 User Space (包含了 World 变换)
                // 这样 clip_rect 才能正确对应屏幕上的位置
                self.current_render.SetTransform(&pre_clip_transform);

                // 调用智能裁剪 helper
                // 它会自动判断是否有旋转：
                // - 无旋转：使用 PushAxisAlignedClip (快)
                // - 有旋转：使用 PushLayer + GeometricMask (正确，且复用 Layer 池)
                self.with_content_clip(&clip_rect, draw_content)?;
            } else {
                // 无需裁剪，直接绘制
                draw_content(self)?;
            }

            // ---- 6. 最终恢复 ----
            self.current_render.SetTransform(&old_transform);
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
        unsafe {
            // 1. 获取资源
            // 注意：这里需确保 handle 对应的 SVG 存在
            let svg_doc = handle.raw();

            // 获取 DeviceContext5
            let dc5: ID2D1DeviceContext5 = self.current_render.cast()?;

            // 2. 获取 SVG 原始尺寸
            // get_svg_size 需要是你之前实现的那个能正确读取 viewBox 或 width/height 的版本
            let (src_w, src_h) = get_svg_size(svg_doc).unwrap_or((100.0, 100.0));

            // 确定目标尺寸
            let target_w = width.unwrap_or(src_w);
            let target_h = height.unwrap_or(src_h);

            // 3. 解析 Options
            let (scale_mode, user_transform, shadow) = match options {
                Some(opt) => (
                    opt.scale_mode.unwrap_or(ScaleMode::None),
                    opt.transform,
                    opt.shadow,
                ),
                None => (ScaleMode::None, None, None),
            };

            // 4. 计算 布局矩阵 (Layout Matrix)
            // 这里的逻辑只负责计算 "如何把 SVG 放到目标框里"
            // 不做具体的坐标加减，只生成 Scale 和 Translation 矩阵
            let (scale_x, scale_y, offset_x, offset_y) = match scale_mode {
                ScaleMode::None => (1.0, 1.0, x, y), // 保持原样，只移动到 x,y
                ScaleMode::Stretch => (target_w / src_w, target_h / src_h, x, y), // 同 Fill
                ScaleMode::Fit => {
                    let ratio = (target_w / src_w).min(target_h / src_h);
                    // 居中偏移量
                    let dx = x + (target_w - src_w * ratio) / 2.0;
                    let dy = y + (target_h - src_h * ratio) / 2.0;
                    (ratio, ratio, dx, dy)
                }
                ScaleMode::Cover => {
                    let ratio = (target_w / src_w).max(target_h / src_h);
                    let dx = x + (target_w - src_w * ratio) / 2.0;
                    let dy = y + (target_h - src_h * ratio) / 2.0;
                    (ratio, ratio, dx, dy)
                }
                // 如果有 ScaleMode::Center，逻辑类似 Fit 但 scale 为 1.0
                _ => (1.0, 1.0, x, y),
            };

            // 布局矩阵：先缩放 SVG，再移动到目标位置 (注意：Direct2D 矩阵是 Scale * Translate)
            let layout_matrix =
                Matrix3x2::scale(scale_x, scale_y) * Matrix3x2::translation(offset_x, offset_y);

            // 5. 获取 用户矩阵 (User Transform)
            let user_matrix = if let Some(t) = user_transform {
                t.into_transform()
            } else {
                Matrix3x2::identity()
            };

            // 6. 获取 上下文矩阵 (Context Transform - 如父级滚动偏移)
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            // 7. 组合 最终矩阵 (Final Transform)
            // 顺序至关重要：SVG Local -> Layout(摆放) -> User(旋转/缩放) -> World(屏幕)
            let final_transform = layout_matrix * user_matrix * old_transform;

            // 8. 裁剪处理 (Cover 模式需要裁剪超出部分)
            // 注意：裁剪框是基于 (x,y,w,h) 的，这是在 User Transform 之前的逻辑坐标
            // 但如果 User Transform 做了旋转，轴对齐裁剪(AxisAlignedClip)可能就不够了。
            // 为了简单起见，这里假设裁剪是在 Layout 之后，User 之前？
            // 实际上，如果旋转了，通常希望裁剪框跟着旋转。
            // 但 PushAxisAlignedClip 只能切矩形。
            // 如果需要完美裁剪，应该用 PushLayer。这里暂保留原有逻辑，仅在 Cover 时裁剪目标区域。
            let need_clip = matches!(scale_mode, ScaleMode::Cover);
            if need_clip {
                // 裁剪必须应用在 "User Transform 之后" 的空间吗？
                // 不，PushAxisAlignedClip 受当前 Transform 影响。
                // 这是一个复杂点。简单做法：先不裁剪，或者仅在无旋转时裁剪有效。
                // 或者：使用 PushLayer 配合几何掩码（开销大）。

                // 暂且使用简单的矩形裁剪，注意这在旋转时可能会切成奇怪的形状
                let clip_rect = D2D_RECT_F {
                    left: x,
                    top: y,
                    right: x + target_w,
                    bottom: y + target_h,
                };
                // 这里的 clip_rect 是在 layout 空间定义的，所以我们需要把 Transform 设置为
                // "不包含 Layout 偏移" 的状态吗？
                // Direct2D 的 Clip 是受 SetTransform 影响的。
                // 鉴于复杂性，如果只是为了 Cover 效果，通常是在 CommandList 内部做，或者接受旋转后裁剪框也旋转。
                // 这里我们先应用最终矩阵，再 Clip，意味着 Clip 矩形也会被旋转。这是符合直觉的（相框带着照片转）。

                // 为了让 Clip 生效位置正确，我们需要应用 old_transform * user_matrix ?
                // 不，直接在最终状态下 PushClip 即可，只要 clip_rect 也是被变换过的逻辑坐标。
                // 但是 clip_rect 是 (x,y)，这是在 Layout 之后 User 之前的坐标。
                // 这块比较绕，如果遇到 Cover 裁剪位置不对，需要改用 Layer。

                // 修正：为了让 clip_rect 正确对应到屏幕像素，我们暂时只在 options 为 None 时启用裁剪
                // 或者忽略 transform。为了代码稳健，这里先注释掉 Clip，或者你只在无旋转时开启。
                self.current_render
                    .PushAxisAlignedClip(&clip_rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            }

            // 应用最终矩阵
            // 注意：这步放在 Clip 之前还是之后取决于 Clip 坐标系。
            // 通常：SetTransform -> PushClip -> Draw
            // 我们的 clip_rect 是 (x,y)，这是 Layout 后的坐标。
            // 如果 final_transform 包含了 layout_matrix，那么 (0,0) 就变到了 (x,y)。
            // 所以此时 PushClip 应该切 (0,0, target_w, target_h) 吗？
            // 不，layout_matrix 包含了 translation。
            // 如果我们 SetTransform(final)，那么原点 (0,0) 就被移到了 (x,y) 并缩放了。
            // 此时如果 DrawSvg (它画在 0,0 到 src_w, src_h)，它会出现在正确位置。
            // 如果我们要 Clip，应该在 SetTransform 之后，PushClip(0, 0, src_w, src_h) ?
            // 不，Cover 模式下 src 被放大了。

            //为了避免 Clip 的坐标系地狱，建议：绘制 SVG 不做系统级 Clip，除非使用 Layer。
            // 或者简单点：不 SetTransform，而是用 Layer 做变换。

            // 回到你的需求：只修复 Transform 跑偏的问题。
            // 我们先只应用 SetTransform，Clip 暂时存疑（你可以根据实际效果决定是否开启）。

            // 应用最终变换：此时画布坐标系已经到了目标位置、且应用了旋转
            self.current_render.SetTransform(&final_transform);

            // 9. 绘制流程
            match shadow {
                Some(shadow_opts) => {
                    // 获取缓存的可变借用
                    let mut cache_access = handle.shadow_cache.lock();

                    // === 懒加载缓存 (Lazy Init) ===
                    if cache_access.is_none() {
                        // A. 录制形状 (只做一次)
                        let command_list = dc5.CreateCommandList()?;
                        let old_target = dc5.GetTarget().ok();

                        dc5.SetTarget(&command_list);
                        dc5.SetTransform(&Matrix3x2::identity()); // 归一化坐标
                        dc5.DrawSvgDocument(handle.raw()); // 录制
                        command_list.Close()?;
                        dc5.SetTarget(old_target.as_ref());

                        // B. 创建专用 Effect (只做一次)
                        // 注意：必须创建一个新的 Effect 给这个 SVG 独享
                        let shadow_effect = self.current_render.CreateEffect(&CLSID_D2D1Shadow)?;
                        shadow_effect.SetInput(0, &command_list, true); // 绑定录像带

                        // C. 存入缓存
                        *cache_access = Some(SvgShadowCache {
                            command_list,
                            shadow_effect,
                            last_blur_radius: -1.0, // 强制更新
                        });
                    }

                    // === 渲染缓存 ===
                    // 这里 unwrap 是安全的，因为上面刚刚保证了初始化
                    let Some(cache) = cache_access.as_mut() else {
                        unreachable!();
                    };

                    // A. 更新参数
                    if (cache.last_blur_radius - shadow_opts.blur_radius).abs() > f32::EPSILON {
                        cache.shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                            D2D1_PROPERTY_TYPE_FLOAT,
                            &shadow_opts.blur_radius.to_ne_bytes(),
                        )?;
                        cache.last_blur_radius = shadow_opts.blur_radius;
                    }

                    let color = shadow_opts.color.as_d2d_color();
                    cache.shadow_effect.SetValue(
                        D2D1_SHADOW_PROP_COLOR.0 as u32,
                        D2D1_PROPERTY_TYPE_VECTOR4,
                        slice::from_raw_parts(
                            &color as *const _ as *const u8,
                            size_of::<D2D1_COLOR_F>(),
                        ),
                    )?;

                    // B. 绘制阴影
                    let offset = Vector2 {
                        X: shadow_opts.offset_x,
                        Y: shadow_opts.offset_y,
                    };
                    dc5.DrawImage(
                        &cache.shadow_effect.cast::<ID2D1Image>()?,
                        Some(&offset),
                        None,
                        D2D1_INTERPOLATION_MODE_LINEAR,
                        D2D1_COMPOSITE_MODE_SOURCE_OVER,
                    );

                    // C. 绘制本体 (直接画 command_list 也是一样的，既然有了就用它)
                    dc5.DrawImage(
                        &cache.command_list.cast::<ID2D1Image>()?,
                        None,
                        None,
                        D2D1_INTERPOLATION_MODE_LINEAR,
                        D2D1_COMPOSITE_MODE_SOURCE_OVER,
                    );
                }
                None => {
                    // B. 无阴影：直接设置变换并绘制
                    dc5.DrawSvgDocument(svg_doc);
                }
            }

            // 10. 恢复
            if need_clip {
                self.current_render.PopAxisAlignedClip();
            }
            self.current_render.SetTransform(&old_transform);

            Ok(())
        }
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
        // 1. 确保 Format 是最新的 (Lazy Rebuild)
        self.rebuild_text_format(text_format)?;

        unsafe {
            // 2. 获取资源
            // 关键：使用 clone() (AddRef) 将接口指针从 self 的生命周期中剥离。
            // 这样后续调用 self.get_or_create_shadow_effect() (&mut self) 时就不会报错。
            let d2d_format = text_format.raw();
            let main_brush = brush.raw();

            // 预处理文本 (转 UTF-16)
            let text_utf16 = encode_unicode(text);

            // 3. 处理 矩阵变换 (Transform)
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            // 计算最终变换矩阵
            let final_transform = if let Some(opts) = options {
                if let Some(t) = opts.transform {
                    t.into_transform() * old_transform
                } else {
                    old_transform
                }
            } else {
                old_transform
            };

            // 应用最终矩阵
            self.current_render.SetTransform(&final_transform);

            // 定义标准位置的矩形
            let layout_rect = D2D_RECT_F {
                left,
                top,
                right: left + width,
                bottom: top + height,
            };

            // 4. 处理 阴影 (Shadow)
            if let Some(opts) = options {
                if let Some(shadow) = opts.shadow {
                    let shadow_color = shadow.color.as_d2d_color();

                    // --- 分支 A: 真·模糊阴影 (Quality Path) ---
                    if shadow.blur_radius > 0.0 {
                        // 尝试获取 DC5 接口
                        if let Ok(dc5) = self.current_render.cast::<ID2D1DeviceContext5>() {
                            // 1. 录制文本形状
                            let command_list = dc5.CreateCommandList()?;
                            let old_target = dc5.GetTarget().ok();

                            dc5.SetTarget(&command_list);
                            // 录制时使用 Identity，位置由 DrawText 的 layout_rect 决定
                            // 注意：这里我们是在 final_transform 的坐标系下“原地”录制，
                            // 所以录制环境的 Transform 设为 Identity，内容位置靠 rect 控制。
                            dc5.SetTransform(&Matrix3x2::identity());

                            // 设置纯色画笔录制遮罩
                            self.scratch_brush.SetColor(&D2D1_COLOR_F {
                                r: 0.,
                                g: 0.,
                                b: 0.,
                                a: 1.,
                            });

                            let svg_glyph_style = dc5.CreateSvgGlyphStyle()?;

                            dc5.DrawText(
                                &text_utf16,
                                d2d_format,
                                &layout_rect,
                                &self.scratch_brush,
                                &svg_glyph_style, // svgglyphstyle
                                0,                // colorpaletteindex
                                D2D1_DRAW_TEXT_OPTIONS_NONE,
                                DWRITE_MEASURING_MODE_NATURAL,
                            );

                            command_list.Close()?;
                            dc5.SetTarget(old_target.as_ref());

                            // 2. 绘制特效
                            // 恢复主变换
                            dc5.SetTransform(&final_transform);

                            let shadow_effect = self.get_or_create_shadow_effect()?; // 无需 clone, 直接拿 &Effect

                            shadow_effect.SetInput(0, &command_list, true);
                            shadow_effect.SetValue(
                                D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                                D2D1_PROPERTY_TYPE_FLOAT,
                                &shadow.blur_radius.to_ne_bytes(),
                            )?;
                            shadow_effect.SetValue(
                                D2D1_SHADOW_PROP_COLOR.0 as u32,
                                D2D1_PROPERTY_TYPE_VECTOR4,
                                slice::from_raw_parts(
                                    &shadow_color as *const _ as *const u8,
                                    size_of::<D2D1_COLOR_F>(),
                                ),
                            )?;

                            let offset = Vector2 {
                                X: shadow.offset_x,
                                Y: shadow.offset_y,
                            };

                            dc5.DrawImage(
                                &shadow_effect.cast::<ID2D1Image>()?,
                                Some(&offset),
                                None,
                                D2D1_INTERPOLATION_MODE_LINEAR,
                                D2D1_COMPOSITE_MODE_SOURCE_OVER,
                            );
                        }
                    }
                    // --- 分支 B: 硬阴影 (Fast Path) ---
                    else {
                        self.scratch_brush.SetColor(&shadow_color);

                        // 计算偏移后的矩形
                        let shadow_rect = D2D_RECT_F {
                            left: layout_rect.left + shadow.offset_x,
                            top: layout_rect.top + shadow.offset_y,
                            right: layout_rect.right + shadow.offset_x,
                            bottom: layout_rect.bottom + shadow.offset_y,
                        };

                        // 直接使用 DrawText 绘制偏移后的文本
                        self.current_render.DrawText(
                            &text_utf16,
                            d2d_format,
                            &shadow_rect,
                            &self.scratch_brush,
                            D2D1_DRAW_TEXT_OPTIONS_NONE,
                            DWRITE_MEASURING_MODE_NATURAL,
                        );
                    }
                }
            }

            // 5. 绘制 主文本 (Main Text)
            self.current_render.DrawText(
                &text_utf16,
                d2d_format,
                &layout_rect,
                main_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );

            // 6. 恢复矩阵
            self.current_render.SetTransform(&old_transform);

            Ok(())
        }
    }

    // ---------------------------------------------------------
    // 3. draw_path: 绘制路径描边
    // ---------------------------------------------------------
    fn draw_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        stroke_width: f32,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        unsafe {
            // 1. 获取资源
            let main_brush = brush.raw();

            // 2. 构建几何体 (注意：为了性能，这一步最好也能缓存，但这是另一个话题)
            let geometry = self.build_geometry(path)?;

            // 3. 处理 矩阵变换
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            let final_transform = if let Some(opts) = options {
                if let Some(t) = opts.transform {
                    t.into_transform() * old_transform
                } else {
                    old_transform
                }
            } else {
                old_transform
            };

            self.current_render.SetTransform(&final_transform);

            // 4. 处理 阴影
            if let Some(opts) = options {
                if let Some(shadow) = opts.shadow {
                    let shadow_color = shadow.color.as_d2d_color();

                    // --- 分支 A: 真·模糊阴影 (LRU 缓存版) ---
                    if shadow.blur_radius > 0.0 {
                        // 1. 计算 Hash Key (Path + StrokeWidth)
                        // 假设 path 实现了 Hash，如果没有，你需要给 path 加一个 id 字段
                        let cache_key = compute_path_cache_key(path, stroke_width, true);

                        // 2. 查缓存
                        let cached_cl = self.lur.get(&cache_key).cloned();

                        // 3. 获取或录制
                        let command_list = if let Some(cl) = cached_cl {
                            cl
                        } else {
                            // === 录制 ===
                            let dc5: ID2D1DeviceContext5 = self.current_render.cast()?;
                            let new_list = dc5.CreateCommandList()?;
                            let old_target = dc5.GetTarget().ok();

                            dc5.SetTarget(&new_list);
                            // 归一化录制
                            dc5.SetTransform(&Matrix3x2::identity());

                            self.scratch_brush.SetColor(&D2D1_COLOR_F {
                                r: 0.,
                                g: 0.,
                                b: 0.,
                                a: 1.,
                            });

                            // 录制 Geometry
                            dc5.DrawGeometry(&geometry, &self.scratch_brush, stroke_width, None);

                            new_list.Close()?;
                            dc5.SetTarget(old_target.as_ref());

                            // 存缓存
                            self.lur.put(cache_key, new_list.clone());
                            new_list
                        };

                        // === 绘制特效 ===
                        // 恢复变换
                        self.current_render.SetTransform(&final_transform);

                        let shadow_effect = self.get_or_create_shadow_effect()?;
                        shadow_effect.SetInput(0, &command_list, true);

                        shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                            D2D1_PROPERTY_TYPE_FLOAT,
                            &shadow.blur_radius.to_ne_bytes(),
                        )?;
                        shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_COLOR.0 as u32,
                            D2D1_PROPERTY_TYPE_VECTOR4,
                            slice::from_raw_parts(
                                &shadow_color as *const _ as *const u8,
                                size_of::<D2D1_COLOR_F>(),
                            ),
                        )?;

                        // 计算偏移矩阵
                        let shadow_draw_matrix =
                            Matrix3x2::translation(shadow.offset_x, shadow.offset_y)
                                * final_transform;

                        self.current_render.SetTransform(&shadow_draw_matrix);

                        self.current_render.DrawImage(
                            &shadow_effect.cast::<ID2D1Image>()?,
                            None,
                            None,
                            D2D1_INTERPOLATION_MODE_LINEAR,
                            D2D1_COMPOSITE_MODE_SOURCE_OVER,
                        );

                        self.current_render.SetTransform(&final_transform);
                    }
                    // --- 分支 B: 硬阴影 ---
                    else {
                        self.scratch_brush.SetColor(&shadow_color);
                        let shadow_matrix =
                            Matrix3x2::translation(shadow.offset_x, shadow.offset_y)
                                * final_transform;

                        self.current_render.SetTransform(&shadow_matrix);

                        self.current_render.DrawGeometry(
                            &geometry,
                            &self.scratch_brush,
                            stroke_width,
                            None,
                        );

                        self.current_render.SetTransform(&final_transform);
                    }
                }
            }

            // 5. 绘制主体
            self.current_render
                .DrawGeometry(&geometry, main_brush, stroke_width, None);

            // 6. 恢复
            self.current_render.SetTransform(&old_transform);

            Ok(())
        }
    }

    // ---------------------------------------------------------
    // 4. fill_path: 填充路径
    // ---------------------------------------------------------
    fn fill_path(
        &mut self,
        path: &Path,
        brush: &Self::BrushHandle,
        options: Option<&PathDrawOptions>,
    ) -> Result<(), Self::Error> {
        unsafe {
            // 1. 获取资源
            let main_brush = brush.raw();

            // 2. 构建几何体
            let geometry = self.build_geometry(path)?;

            // 3. 处理 矩阵变换
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            let final_transform = if let Some(opts) = options {
                if let Some(t) = opts.transform {
                    t.into_transform() * old_transform
                } else {
                    old_transform
                }
            } else {
                old_transform
            };

            self.current_render.SetTransform(&final_transform);

            // 4. 处理 阴影
            if let Some(opts) = options {
                if let Some(shadow) = opts.shadow {
                    let shadow_color = shadow.color.as_d2d_color();

                    // --- 分支 A: 真·模糊阴影 (LRU 缓存版) ---
                    if shadow.blur_radius > 0.0 {
                        // 1. 计算 Hash Key (Path Fill)
                        let cache_key = compute_path_cache_key(path, 0.0, false);

                        // 2. 查缓存
                        let cached_cl = self.lur.get(&cache_key).cloned();

                        // 3. 获取或录制
                        let command_list = if let Some(cl) = cached_cl {
                            cl
                        } else {
                            // === 录制 ===
                            let dc5: ID2D1DeviceContext5 = self.current_render.cast()?;
                            let new_list = dc5.CreateCommandList()?;
                            let old_target = dc5.GetTarget().ok();

                            dc5.SetTarget(&new_list);
                            dc5.SetTransform(&Matrix3x2::identity());

                            self.scratch_brush.SetColor(&D2D1_COLOR_F {
                                r: 0.,
                                g: 0.,
                                b: 0.,
                                a: 1.,
                            });

                            // 录制 Geometry Fill
                            dc5.FillGeometry(&geometry, &self.scratch_brush, None);

                            new_list.Close()?;
                            dc5.SetTarget(old_target.as_ref());

                            // 存缓存
                            self.lur.put(cache_key, new_list.clone());
                            new_list
                        };

                        // === 绘制特效 ===
                        self.current_render.SetTransform(&final_transform);

                        let shadow_effect = self.get_or_create_shadow_effect()?;
                        shadow_effect.SetInput(0, &command_list, true);

                        shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                            D2D1_PROPERTY_TYPE_FLOAT,
                            &shadow.blur_radius.to_ne_bytes(),
                        )?;
                        shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_COLOR.0 as u32,
                            D2D1_PROPERTY_TYPE_VECTOR4,
                            slice::from_raw_parts(
                                &shadow_color as *const _ as *const u8,
                                size_of::<D2D1_COLOR_F>(),
                            ),
                        )?;

                        let shadow_draw_matrix =
                            Matrix3x2::translation(shadow.offset_x, shadow.offset_y)
                                * final_transform;

                        self.current_render.SetTransform(&shadow_draw_matrix);

                        self.current_render.DrawImage(
                            &shadow_effect.cast::<ID2D1Image>()?,
                            None,
                            None,
                            D2D1_INTERPOLATION_MODE_LINEAR,
                            D2D1_COMPOSITE_MODE_SOURCE_OVER,
                        );

                        self.current_render.SetTransform(&final_transform);
                    }
                    // --- 分支 B: 硬阴影 ---
                    else {
                        self.scratch_brush.SetColor(&shadow_color);
                        let shadow_matrix =
                            Matrix3x2::translation(shadow.offset_x, shadow.offset_y)
                                * final_transform;

                        self.current_render.SetTransform(&shadow_matrix);

                        self.current_render
                            .FillGeometry(&geometry, &self.scratch_brush, None);

                        self.current_render.SetTransform(&final_transform);
                    }
                }
            }

            // 5. 绘制 主体
            self.current_render
                .FillGeometry(&geometry, main_brush, None);

            // 6. 恢复
            self.current_render.SetTransform(&old_transform);

            Ok(())
        }
    }

    // ---------------------------------------------------------
    // 1. draw_quad: 绘制矩形边框 (描边)
    // ---------------------------------------------------------
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
        // 1. 获取资源
        let main_brush = brush.raw();

        let rect = D2D_RECT_F {
            left,
            top,
            right: left + width,
            bottom: top + height,
        };

        unsafe {
            // 2. 变换
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            let final_transform = if let Some(opts) = options {
                if let Some(t) = opts.transform {
                    t.into_transform() * old_transform
                } else {
                    old_transform
                }
            } else {
                old_transform
            };

            self.current_render.SetTransform(&final_transform);

            // 3. 阴影处理
            if let Some(opts) = options {
                if let Some(shadow) = opts.shadow {
                    let shadow_color = shadow.color.as_d2d_color();

                    // --- 分支 A: 真·模糊阴影 (LRU 缓存优化版) ---
                    if shadow.blur_radius > 0.0 {
                        // 1. 计算 Hash Key
                        let cache_key = compute_stroke_rect_hash(width, height, border_width);

                        // 2. 尝试从缓存获取
                        let cached_cl = self.lur.get(&cache_key).cloned();

                        // 3. 准备 CommandList
                        let command_list = if let Some(cl) = cached_cl {
                            cl
                        } else {
                            // === 开始录制 ===
                            // 只有录制时才需要尝试转换到 DC5 (如果你的 CreateCommandList 依赖它)
                            // 注意：其实 ID2D1DeviceContext 也有 CreateCommandList，看你具体绑定
                            let dc5: ID2D1DeviceContext5 = self.current_render.cast()?;
                            let new_list = dc5.CreateCommandList()?;
                            let old_target = dc5.GetTarget().ok();

                            dc5.SetTarget(&new_list);

                            // 归一化录制
                            dc5.SetTransform(&Matrix3x2::identity());

                            self.scratch_brush.SetColor(&D2D1_COLOR_F {
                                r: 0.,
                                g: 0.,
                                b: 0.,
                                a: 1.,
                            });

                            // 在 (0,0) 绘制标准形状
                            let local_rect = D2D_RECT_F {
                                left: 0.0,
                                top: 0.0,
                                right: width,
                                bottom: height,
                            };

                            dc5.DrawRectangle(&local_rect, &self.scratch_brush, border_width, None);

                            new_list.Close()?;
                            dc5.SetTarget(old_target.as_ref());

                            // 存入缓存
                            self.lur.put(cache_key, new_list.clone());
                            new_list
                        }; // <--- dc5 在这里生命周期结束，这是对的

                        // === 绘制特效 ===
                        // 修正点：下面全部改用 self.current_render

                        // 恢复变换 (因为录制时可能动了，虽然是在 commandlist 内部动的，但安全起见)
                        self.current_render.SetTransform(&final_transform);

                        let shadow_effect = self.get_or_create_shadow_effect()?;
                        shadow_effect.SetInput(0, &command_list, true);

                        shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                            D2D1_PROPERTY_TYPE_FLOAT,
                            &shadow.blur_radius.to_ne_bytes(),
                        )?;
                        shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_COLOR.0 as u32,
                            D2D1_PROPERTY_TYPE_VECTOR4,
                            slice::from_raw_parts(
                                &shadow_color as *const _ as *const u8,
                                size_of::<D2D1_COLOR_F>(),
                            ),
                        )?;

                        // 计算播放矩阵
                        let shadow_draw_matrix =
                            Matrix3x2::translation(left + shadow.offset_x, top + shadow.offset_y)
                                * final_transform;

                        self.current_render.SetTransform(&shadow_draw_matrix);

                        // 绘制
                        self.current_render.DrawImage(
                            &shadow_effect.cast::<ID2D1Image>()?,
                            None,
                            None,
                            D2D1_INTERPOLATION_MODE_LINEAR,
                            D2D1_COMPOSITE_MODE_SOURCE_OVER,
                        );

                        // 恢复变换用于绘制主体
                        self.current_render.SetTransform(&final_transform);
                    }
                    // --- 分支 B: 硬阴影 ---
                    else {
                        self.scratch_brush.SetColor(&shadow_color);

                        let shadow_rect = D2D_RECT_F {
                            left: rect.left + shadow.offset_x,
                            top: rect.top + shadow.offset_y,
                            right: rect.right + shadow.offset_x,
                            bottom: rect.bottom + shadow.offset_y,
                        };

                        self.current_render.DrawRectangle(
                            &shadow_rect,
                            &self.scratch_brush,
                            border_width,
                            None,
                        );
                    }
                }
            }

            // 4. 绘制 主体
            self.current_render
                .DrawRectangle(&rect, main_brush, border_width, None);

            // 5. 恢复
            self.current_render.SetTransform(&old_transform);
        }
        Ok(())
    }

    // ---------------------------------------------------------
    // 2. fill_quad: 填充矩形 (带圆角支持)
    // ---------------------------------------------------------
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
        // 几何参数 (用于主体的绘制，位置是绝对的)
        let rect = D2D_RECT_F {
            left,
            top,
            right: left + width,
            bottom: top + height,
        };
        let radius = corner_radius.unwrap_or(0.0);
        let is_rounded = radius > 0.0;

        // 预计算 RoundedRect (用于主体绘制)
        let rounded = if is_rounded {
            Some(D2D1_ROUNDED_RECT {
                rect,
                radiusX: radius,
                radiusY: radius,
            })
        } else {
            None
        };

        unsafe {
            // 2. 变换
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            let final_transform = if let Some(opts) = options {
                if let Some(t) = opts.transform {
                    t.into_transform() * old_transform
                } else {
                    old_transform
                }
            } else {
                old_transform
            };

            self.current_render.SetTransform(&final_transform);

            // 3. 阴影处理
            if let Some(opts) = options {
                if let Some(shadow) = opts.shadow {
                    let shadow_color = shadow.color.as_d2d_color();

                    // --- 分支 A: 真·模糊阴影 (带 LRU 缓存优化) ---
                    if shadow.blur_radius > 0.0 {
                        // 1. 计算 Hash Key (只跟尺寸有关，跟位置无关)
                        let cache_key = compute_quad_hash(width, height, radius);

                        // 2. 尝试从缓存获取 (Clone 出接口指针，解开借用)
                        let cached_cl = self.lur.get(&cache_key).cloned();

                        // 3. 准备 CommandList (命中则复用，未命中则录制)
                        let command_list = if let Some(cl) = cached_cl {
                            cl
                        } else {
                            // === 开始录制 ===
                            // 必须获取 DC5 接口
                            let dc5: ID2D1DeviceContext5 = self.current_render.cast()?;
                            let new_list = dc5.CreateCommandList()?;
                            let old_target = dc5.GetTarget().ok();

                            dc5.SetTarget(&new_list);

                            // 【关键点】：归一化录制。在 (0,0) 原点录制标准形状。
                            dc5.SetTransform(&Matrix3x2::identity());

                            self.scratch_brush.SetColor(&D2D1_COLOR_F {
                                r: 0.,
                                g: 0.,
                                b: 0.,
                                a: 1.,
                            });

                            // 构造局部的 rect (0, 0, w, h)
                            let local_rect = D2D_RECT_F {
                                left: 0.0,
                                top: 0.0,
                                right: width,
                                bottom: height,
                            };

                            if is_rounded {
                                let local_rounded = D2D1_ROUNDED_RECT {
                                    rect: local_rect,
                                    radiusX: radius,
                                    radiusY: radius,
                                };
                                dc5.FillRoundedRectangle(&local_rounded, &self.scratch_brush);
                            } else {
                                dc5.FillRectangle(&local_rect, &self.scratch_brush);
                            }

                            new_list.Close()?;
                            dc5.SetTarget(old_target.as_ref());

                            // 存入缓存
                            self.lur.put(cache_key, new_list.clone());
                            new_list
                        };

                        // === 绘制特效 ===
                        // 复用 Shadow Effect (不需要每帧 CreateEffect)
                        let shadow_effect = self.get_or_create_shadow_effect()?;
                        shadow_effect.SetInput(0, &command_list, true); // true = 允许缓存

                        shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                            D2D1_PROPERTY_TYPE_FLOAT,
                            &shadow.blur_radius.to_ne_bytes(),
                        )?;
                        shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_COLOR.0 as u32,
                            D2D1_PROPERTY_TYPE_VECTOR4,
                            slice::from_raw_parts(
                                &shadow_color as *const _ as *const u8,
                                size_of::<D2D1_COLOR_F>(),
                            ),
                        )?;

                        // 【关键点】：计算播放矩阵
                        // 我们录制的是 (0,0)，现在要把阴影移到 (left+offset, top+offset)
                        // 公式：Translation(阴影位置) * final_transform
                        let shadow_draw_matrix =
                            Matrix3x2::translation(left + shadow.offset_x, top + shadow.offset_y)
                                * final_transform;

                        self.current_render.SetTransform(&shadow_draw_matrix);

                        // 绘制 (Offset 传 None，因为已经在 Matrix 里处理了)
                        self.current_render.DrawImage(
                            &shadow_effect.cast::<ID2D1Image>()?,
                            None,
                            None,
                            D2D1_INTERPOLATION_MODE_LINEAR,
                            D2D1_COMPOSITE_MODE_SOURCE_OVER,
                        );

                        // 恢复变换，准备绘制主体
                        self.current_render.SetTransform(&final_transform);
                    }
                    // --- 分支 B: 硬阴影 (Fast Path) ---
                    else {
                        self.scratch_brush.SetColor(&shadow_color);
                        let offset_rect = D2D_RECT_F {
                            left: rect.left + shadow.offset_x,
                            top: rect.top + shadow.offset_y,
                            right: rect.right + shadow.offset_x,
                            bottom: rect.bottom + shadow.offset_y,
                        };

                        if let Some(r) = &rounded {
                            let offset_rounded = D2D1_ROUNDED_RECT {
                                rect: offset_rect,
                                radiusX: r.radiusX,
                                radiusY: r.radiusY,
                            };
                            self.current_render
                                .FillRoundedRectangle(&offset_rounded, &self.scratch_brush);
                        } else {
                            self.current_render
                                .FillRectangle(&offset_rect, &self.scratch_brush);
                        }
                    }
                }
            }

            // 4. 绘制主体 (保持原样，直接用 rect 绘制在 left/top)
            if let Some(r) = &rounded {
                self.current_render.FillRoundedRectangle(r, brush.raw());
            } else {
                self.current_render.FillRectangle(&rect, brush.raw());
            }

            // 5. 恢复
            self.current_render.SetTransform(&old_transform);
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
        unsafe {
            // 1. 计算变换矩阵
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            let local_transform = if let Some(t) = transform {
                t.into_transform()
            } else {
                Matrix3x2::identity()
            };

            // 最终矩阵 = 局部 * 全局
            let final_transform = local_transform * old_transform;

            // 2. 创建几何体 (Geometry)
            // 我们需要 Geometry 来计算 Bounds 和执行裁剪
            let rect = D2D_RECT_F {
                left,
                top,
                right: left + width,
                bottom: top + height,
            };

            let geometry: ID2D1Geometry = if corner_radius > 0.0 {
                let rounded = D2D1_ROUNDED_RECT {
                    rect,
                    radiusX: corner_radius,
                    radiusY: corner_radius,
                };
                let g = RenderFactory::get()
                    .factory
                    .CreateRoundedRectangleGeometry(&rounded)?;
                g.cast()?
            } else {
                let g = RenderFactory::get()
                    .factory
                    .CreateRectangleGeometry(&rect)?;
                g.cast()?
            };

            // 3. 获取模糊画笔 (这一步包含了复杂的 Snapshot 逻辑)
            let (blur_brush, _) =
                self.create_blur_brush_for_geometry(&geometry, blur_radius, &final_transform)?;

            // 4. 创建变换后的几何体 (Transformed Geometry)
            // 为了保证像素 1:1 对齐，我们在 Identity 空间进行绘制
            let transformed_geometry = RenderFactory::get()
                .factory
                .CreateTransformedGeometry(&geometry, &final_transform)?;

            // 5. 切换到 Identity 空间进行绘制
            self.current_render.SetTransform(&Matrix3x2::identity());

            self.current_render
                .FillGeometry(&transformed_geometry, &blur_brush, None);

            // 6. 恢复变换
            self.current_render.SetTransform(&old_transform);

            Ok(())
        }
    }

    fn blur_path(
        &mut self,
        path: &Path,
        blur_radius: f32,
        transform: Option<&Transform2D>,
    ) -> Result<(), Self::Error> {
        unsafe {
            // 1. 计算变换矩阵
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            let local_transform = if let Some(t) = transform {
                t.into_transform()
            } else {
                Matrix3x2::identity()
            };
            let final_transform = local_transform * old_transform;

            // 2. 构建几何体 (复用之前的 build_geometry)
            // build_geometry 返回的是 ID2D1PathGeometry，也是 ID2D1Geometry
            let geometry = self.build_geometry(path)?;
            let geometry_interface = geometry.cast()?;

            // 3. 获取模糊画笔
            let (blur_brush, _) = self.create_blur_brush_for_geometry(
                &geometry_interface,
                blur_radius,
                &final_transform,
            )?;

            // 4. 创建变换后的几何体
            let transformed_geometry = RenderFactory::get()
                .factory
                .CreateTransformedGeometry(&geometry_interface, &final_transform)?;

            // 5. 切换到 Identity 绘制
            self.current_render.SetTransform(&Matrix3x2::identity());

            self.current_render
                .FillGeometry(&transformed_geometry, &blur_brush, None);

            // 6. 恢复
            self.current_render.SetTransform(&old_transform);

            Ok(())
        }
    }

    fn set_clip(
        &mut self,
        surface_id: Option<&Self::SurfaceId>,
        rect: Option<(f32, f32, f32, f32)>,
    ) -> Result<(), Self::Error> {
        unsafe {
            // 1. 获取 Render Target
            let render_target = match surface_id {
                None => self.current_render.cast::<ID2D1RenderTarget>()?,
                Some(surface_id) => surface_id.raw().cast::<ID2D1RenderTarget>()?,
            };

            match rect {
                // 开启裁剪：只 Push，不 Pop！
                // 状态保留在 D2D 自己的栈里，直到你下次调用 set_clip(None)
                Some((x, y, w, h)) => {
                    let d2d_rect = D2D_RECT_F {
                        left: x,
                        top: y,
                        right: x + w,
                        bottom: y + h,
                    };
                    render_target.PushAxisAlignedClip(&d2d_rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
                }

                // 结束裁剪：执行 Pop
                // 这里假设你之前一定调用过 Some(...)，否则 D2D 可能会报错
                None => {
                    render_target.PopAxisAlignedClip();
                }
            }
        }
        Ok(())
    }

    fn capture_snapshot(
        &mut self,
        rect: Option<(f32, f32, u32, u32)>,
    ) -> Result<Vec<u8>, Self::Error> {
        unsafe {
            // 1. 确定截取区域
            let (x, y, w, h) = match rect {
                Some((x, y, w, h)) => (x as u32, y as u32, w, h),
                None => (0, 0, self.size.width, self.size.height),
            };

            if w == 0 || h == 0 {
                return Ok(Vec::new());
            }

            // 4. 创建 Staging Bitmap
            // 这里的 size 是我们要截取的大小
            let staging_size = D2D_SIZE_U {
                width: w,
                height: h,
            };
            let staging_bitmap = self.current_render.CreateBitmap(
                staging_size,
                None, // 初始不给数据
                0,
                &RenderFactory::get().d2d1_bitmap_properties1,
            )?;

            // 5. 从当前 RenderTarget 复制数据到 Staging Bitmap
            // dest_point: 复制到新位图的 (0,0) 位置
            // src_rect: 从 RenderTarget 的 (x,y) 位置开始复制
            let dest_point = D2D_POINT_2U { x: 0, y: 0 };
            let src_rect = D2D_RECT_U {
                left: x,
                top: y,
                right: x + w,
                bottom: y + h,
            };

            // CopyFromRenderTarget 是关键 API，它能把 GPU 上的内容拉取到 Staging Bitmap
            staging_bitmap.CopyFromRenderTarget(
                Some(&dest_point),
                &self.current_render,
                Some(&src_rect),
            )?;

            // 6. 映射内存进行读取
            let mapped_rect = staging_bitmap.Map(D2D1_MAP_OPTIONS_READ)?;

            // 7. 将数据搬运到 Vec<u8>
            // 注意：mapped_rect.pitch (步长) 可能大于 width * 4，因为有内存对齐填充。
            // 我们需要一行一行地复制，去除填充字节。
            let row_size = (w * 4) as usize; // 假设是 32 位颜色 (RGBA/BGRA)
            let total_size = row_size * h as usize;
            let mut buffer = Vec::with_capacity(total_size);

            let src_ptr = mapped_rect.bits;
            let pitch = mapped_rect.pitch as usize;

            for row in 0..h as usize {
                let start = src_ptr.add(row * pitch);
                // 这是一个 unsafe 的切片读取
                let row_slice = slice::from_raw_parts(start, row_size);

                // 遍历每个像素 (4字节) 进行通道交换
                // Direct2D (BGRA) -> Image Crate (RGBA)
                for pixel in row_slice.chunks_exact(4) {
                    let b = pixel[0];
                    let g = pixel[1];
                    let r = pixel[2];
                    let a = pixel[3];

                    // 重新排列推入 buffer
                    buffer.push(r);
                    buffer.push(g);
                    buffer.push(b);
                    buffer.push(a);
                }
            }

            // 8. 解除映射
            staging_bitmap.Unmap()?;

            Ok(buffer)
        }
    }
}

impl D2DRender {
    // 内部 helper：获取可复用的 Effect
    fn get_or_create_shadow_effect(&mut self) -> Result<ID2D1Effect, D2DBackendError> {
        // 1. 如果已经存在，直接返回引用的引用
        if let Some(effect) = &self.cached_shadow_effect {
            return Ok(effect.clone());
        }

        // 2. 如果不存在，创建新的
        let effect = unsafe {
            let effect = self.current_render.CreateEffect(&CLSID_D2D1Shadow)?;
            self.cached_shadow_effect = Some(effect.clone());
            effect
        };

        Ok(effect)
    }

    fn rebuild_text_format(
        &mut self,
        text_format: &mut D2DTextFormatHandle,
    ) -> Result<(), D2DBackendError> {
        // 1. 极速路径 (Fast Path)
        // 只要没脏，直接返回。不锁 Map，不查 Key，不做任何多余操作。
        // 这让每一帧调用 rebuild 的开销几乎为 0。
        if !text_format.dirty() {
            return Ok(());
        }

        // =========================================================
        // 慢路径 (Slow Path): 只有需要更新时才走这里
        // =========================================================

        // 2. 准备不可变参数
        let family = HSTRING::from(&text_format.font_family_name());
        let (weight, style, stretch) = text_format.map_props();
        let locale = HSTRING::from("en-us");

        // 3. 创建核心对象 (耗时操作，不持有锁)
        let new_text_format = unsafe {
            RenderFactory::get().write_factory.CreateTextFormat(
                PCWSTR(family.as_ptr()),
                None,
                weight,
                style,
                stretch,
                text_format.font_size(),
                PCWSTR(locale.as_ptr()),
            )?
        };

        // 4. 配置可变属性 (耗时操作，不持有锁)
        unsafe {
            // Alignment
            let dw_text_align = match text_format.text_alignment() {
                TextAlignment::Start => DWRITE_TEXT_ALIGNMENT_LEADING,
                TextAlignment::End => DWRITE_TEXT_ALIGNMENT_TRAILING,
                TextAlignment::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
                TextAlignment::Justified => DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
            };
            new_text_format.SetTextAlignment(dw_text_align)?;

            let dw_para_align = match text_format.paragraph_alignment() {
                ParagraphAlignment::Top => DWRITE_PARAGRAPH_ALIGNMENT_NEAR,
                ParagraphAlignment::Center => DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
                ParagraphAlignment::Bottom => DWRITE_PARAGRAPH_ALIGNMENT_FAR,
            };
            new_text_format.SetParagraphAlignment(dw_para_align)?;

            // Wrapping
            let dw_wrap = match text_format.word_wrapping() {
                WordWrapping::NoWrap => DWRITE_WORD_WRAPPING_NO_WRAP,
                WordWrapping::Wrap => DWRITE_WORD_WRAPPING_WRAP,
                WordWrapping::Character => DWRITE_WORD_WRAPPING_CHARACTER,
            };
            new_text_format.SetWordWrapping(dw_wrap)?;

            // Line Height
            if (text_format.line_height() - 1.0).abs() > f32::EPSILON {
                let line_spacing = text_format.font_size() * text_format.line_height();
                let baseline = line_spacing * 0.8;
                new_text_format.SetLineSpacing(
                    DWRITE_LINE_SPACING_METHOD_UNIFORM,
                    line_spacing,
                    baseline,
                )?;
            } else {
                new_text_format.SetLineSpacing(DWRITE_LINE_SPACING_METHOD_DEFAULT, 0.0, 0.0)?;
            }

            // Trimming
            let (granularity, _delimiter) = match text_format.text_trimming() {
                TextTrimming::None => (DWRITE_TRIMMING_GRANULARITY_NONE, 0),
                TextTrimming::Character | TextTrimming::EllipsisChar => {
                    (DWRITE_TRIMMING_GRANULARITY_CHARACTER, 0)
                }
                TextTrimming::Word | TextTrimming::EllipsisWord => {
                    (DWRITE_TRIMMING_GRANULARITY_WORD, 0)
                }
            };
            // 暂不支持省略号对象，传 None
            new_text_format.SetTrimming(
                &DWRITE_TRIMMING {
                    granularity,
                    delimiter: 0,
                    delimiterCount: 0,
                },
                None,
            )?;
        }
        text_format.set_raw(new_text_format);
        text_format.clear_dirty();
        Ok(())
    }

    /// 内部辅助：将 Path 转换为 Direct2D Geometry
    fn build_geometry(&self, path: &Path) -> Result<ID2D1PathGeometry1, D2DBackendError> {
        unsafe {
            // 1. 创建 PathGeometry
            let geometry = RenderFactory::get().factory.CreatePathGeometry()?;

            // 2. 打开 Sink 写入命令
            let sink = geometry.Open()?;

            // 标记当前是否有打开的 Figure (图形段)
            let mut is_figure_started = false;
            // 跟踪当前点位置 (Start point for next command)
            let mut current_point = Vector2 { X: 0.0, Y: 0.0 };

            for cmd in path.commands() {
                match cmd {
                    PathCommand::MoveTo(x, y) => {
                        // 如果之前有未闭合的段，先结束它 (默认 OPEN)
                        if is_figure_started {
                            sink.EndFigure(D2D1_FIGURE_END_OPEN);
                        }
                        // 开始新段
                        let pt = Vector2 { X: *x, Y: *y };
                        sink.BeginFigure(pt, D2D1_FIGURE_BEGIN_FILLED);
                        is_figure_started = true;
                        current_point = pt;
                    }
                    PathCommand::LineTo(x, y) => {
                        if is_figure_started {
                            let pt = Vector2 { X: *x, Y: *y };
                            sink.AddLine(pt);
                            current_point = pt;
                        }
                    }
                    PathCommand::Bezier(points) => {
                        if is_figure_started {
                            // 根据点数判断是二次还是三次贝塞尔
                            if points.len() == 2 {
                                // 二次贝塞尔 (1 控制点 + 1 终点)
                                let bezier = D2D1_QUADRATIC_BEZIER_SEGMENT {
                                    point1: Vector2 {
                                        X: points[0].0,
                                        Y: points[0].1,
                                    }, // Control
                                    point2: Vector2 {
                                        X: points[1].0,
                                        Y: points[1].1,
                                    }, // End
                                };
                                sink.AddQuadraticBezier(&bezier);
                                current_point = bezier.point2;
                            } else if points.len() == 3 {
                                // 三次贝塞尔 (2 控制点 + 1 终点)
                                let bezier = D2D1_BEZIER_SEGMENT {
                                    point1: Vector2 {
                                        X: points[0].0,
                                        Y: points[0].1,
                                    }, // Control 1
                                    point2: Vector2 {
                                        X: points[1].0,
                                        Y: points[1].1,
                                    }, // Control 2
                                    point3: Vector2 {
                                        X: points[2].0,
                                        Y: points[2].1,
                                    }, // End
                                };
                                sink.AddBezier(&bezier);
                                current_point = bezier.point3;
                            } else if points.len() > 3 {
                                // === 无限贝塞尔 (High-Order Bezier) ===
                                // 1. 构建完整的控制点列表 (Start + Points)
                                let mut control_points = Vec::with_capacity(points.len() + 1);
                                control_points.push(current_point);
                                for p in points {
                                    control_points.push(Vector2 { X: p.0, Y: p.1 });
                                }

                                // 2. 离散化 (Discretize)
                                // 步数可以根据曲线长度动态计算，这里暂定固定 100 段
                                let steps = 100;
                                for i in 1..=steps {
                                    let t = i as f32 / steps as f32;
                                    let next_pt = calculate_bezier_point(t, &control_points);
                                    sink.AddLine(next_pt);
                                    current_point = next_pt;
                                }
                            } else {
                                // 点数 < 2 的情况 (不合法)
                                eprintln!("Unsupported Bezier points count: {}", points.len());
                            }
                        }
                    }
                    PathCommand::Close => {
                        if is_figure_started {
                            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                            is_figure_started = false;
                        }
                    }
                }
            }

            // 循环结束后，如果还有未闭合的段，将其结束
            if is_figure_started {
                sink.EndFigure(D2D1_FIGURE_END_OPEN);
            }

            // 3. 关闭 Sink，完成构建
            sink.Close()?;

            Ok(geometry)
        }
    }

    /// 内部通用逻辑：执行截屏、模糊并返回画笔和物理位置
    /// 返回: (模糊后的画笔, 物理包围盒左上角偏移)
    // 内部 helper：获取可复用的 Blur Effect
    fn get_or_create_blur_effect(&mut self) -> Result<ID2D1Effect, D2DBackendError> {
        if let Some(effect) = &self.cached_blur_effect {
            return Ok(effect.clone());
        }

        let effect = unsafe {
            let effect = self.current_render.CreateEffect(&CLSID_D2D1GaussianBlur)?;
            self.cached_blur_effect = Some(effect.clone());
            effect
        };

        Ok(effect)
    }

    // 内部 helper：获取可复用的 Bitmap (Scratch Bitmap)
    fn get_scratch_bitmap(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<ID2D1Bitmap1, D2DBackendError> {
        unsafe {
            let need_create = if let Some(current_bmp) = &self.cached_blur_bitmap {
                let size = current_bmp.GetPixelSize();
                size.width < width || size.height < height
            } else {
                true
            };

            if need_create {
                let pixel_format = self.current_render.GetPixelFormat();
                let mut dpi_x = 0.0;
                let mut dpi_y = 0.0;
                self.current_render.GetDpi(&mut dpi_x, &mut dpi_y);

                let bitmap_props = D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: pixel_format,
                    dpiX: dpi_x,
                    dpiY: dpi_y,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                    colorContext: ManuallyDrop::new(None),
                };

                let new_bmp = self.current_render.CreateBitmap(
                    D2D_SIZE_U { width, height },
                    None,
                    0,
                    &bitmap_props,
                )?;
                self.cached_blur_bitmap = Some(new_bmp);
            }

            Ok(self.cached_blur_bitmap.as_ref().unwrap().clone())
        }
    }

    // 内部 helper：获取可复用的 Crop Effect
    fn get_or_create_crop_effect(&mut self) -> Result<ID2D1Effect, D2DBackendError> {
        if let Some(effect) = &self.cached_crop_effect {
            return Ok(effect.clone());
        }

        let effect = unsafe {
            let effect = self.current_render.CreateEffect(&CLSID_D2D1Crop)?;
            self.cached_crop_effect = Some(effect.clone());
            effect
        };

        Ok(effect)
    }

    /// 内部通用逻辑：执行截屏、模糊并返回画笔和物理位置
    /// 返回: (模糊后的画笔, 物理包围盒左上角偏移)
    fn create_blur_brush_for_geometry(
        &mut self,
        geometry: &ID2D1Geometry,
        blur_radius: f32,
        world_transform: &Matrix3x2,
    ) -> Result<(ID2D1ImageBrush, Vector2), D2DBackendError> {
        unsafe {
            let bounds = geometry.GetBounds(Some(world_transform))?;

            // 转换为整数坐标 (向外取整，确保覆盖所有像素)
            let left_u = bounds.left.floor() as i32;
            let top_u = bounds.top.floor() as i32;
            let right_u = bounds.right.ceil() as i32;
            let bottom_u = bounds.bottom.ceil() as i32;

            let width_u = (right_u - left_u).max(1) as u32;
            let height_u = (bottom_u - top_u).max(1) as u32;

            // 1. 获取复用的 Bitmap
            let snapshot = self.get_scratch_bitmap(width_u, height_u)?;

            // 2. 将屏幕内容拷贝到 Bitmap 的 (0,0) 位置
            let dest_point = D2D_POINT_2U { x: 0, y: 0 };
            let src_rect = D2D_RECT_U {
                left: left_u as u32,
                top: top_u as u32,
                right: (left_u as u32) + width_u,
                bottom: (top_u as u32) + height_u,
            };

            // 注意：CopyFromRenderTarget 会把 src_rect 区域拷贝到 bitmap 的 dest_point
            snapshot.CopyFromRenderTarget(
                Some(&dest_point),
                &self.current_render,
                Some(&src_rect),
            )?;

            // 3. 使用 Crop Effect 裁剪出有效区域 (避免 Blur 采样到右侧/下侧的脏数据)
            let crop_effect = self.get_or_create_crop_effect()?;
            crop_effect.SetInput(0, &snapshot, true);

            let crop_rect = D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: width_u as f32,
                bottom: height_u as f32,
            };
            crop_effect.SetValue(
                D2D1_CROP_PROP_RECT.0 as u32,
                D2D1_PROPERTY_TYPE_VECTOR4,
                slice::from_raw_parts(&crop_rect as *const _ as *const u8, size_of::<D2D_RECT_F>()),
            )?;

            // 4. 获取复用的 Blur Effect，连接到 Crop
            let blur_effect = self.get_or_create_blur_effect()?;

            // 链接: Blur -> Input(Crop output)
            // 注意: GetOutput 会增加引用计数，记得这只是个临时 Image 接口
            let crop_output = crop_effect.GetOutput()?;
            blur_effect.SetInput(0, &crop_output, true);

            // 设置参数 (如果半径变了)
            if (self.last_blur_radius - blur_radius).abs() > 0.001 {
                blur_effect.SetValue(
                    D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION.0 as u32,
                    D2D1_PROPERTY_TYPE_FLOAT,
                    &blur_radius.to_ne_bytes(),
                )?;
                // Border mode 只需要设置一次，但我懒得加 cached_border_mode 了，且开销很小
                let border_mode = D2D1_BORDER_MODE_HARD;
                blur_effect.SetValue(
                    D2D1_GAUSSIANBLUR_PROP_BORDER_MODE.0 as u32,
                    D2D1_PROPERTY_TYPE_ENUM,
                    slice::from_raw_parts(
                        &border_mode as *const _ as *const u8,
                        size_of::<D2D1_BORDER_MODE>(),
                    ),
                )?;
                self.last_blur_radius = blur_radius;
            }

            let output_image: ID2D1Image = blur_effect.GetOutput()?;

            // 4. 创建画笔 (画笔创建比较轻量，可以保留每次创建，或者也想办法缓存？目前先保留创建)
            let brush_props = D2D1_IMAGE_BRUSH_PROPERTIES {
                sourceRectangle: D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: width_u as f32,
                    bottom: height_u as f32,
                },
                extendModeX: D2D1_EXTEND_MODE_CLAMP,
                extendModeY: D2D1_EXTEND_MODE_CLAMP,
                interpolationMode: D2D1_INTERPOLATION_MODE_LINEAR,
            };

            let brush = self
                .current_render
                .CreateImageBrush(&output_image, &brush_props, None)?;

            let brush_transform = Matrix3x2::translation(left_u as f32, top_u as f32);
            brush.SetTransform(&brush_transform);

            Ok((brush, Vector2 { X: 0., Y: 0. }))
        }
    }

    // 内部 Helper：获取一个可用的 Layer
    fn get_layer(&mut self) -> Result<ID2D1Layer, D2DBackendError> {
        if let Some(layer) = self.layer_pool.pop() {
            Ok(layer)
        } else {
            // 池子空了，创建一个新的
            let new_layer = unsafe { self.current_render.CreateLayer(None)? };
            Ok(new_layer)
        }
    }

    // 内部 Helper：归还 Layer
    fn return_layer(&mut self, layer: ID2D1Layer) {
        self.layer_pool.push(layer);
    }

    // 这是一个通用的裁剪包装器
    // rect: 裁剪区域（在当前坐标系下）
    // draw_fn: 具体的绘制逻辑闭包
    fn with_content_clip<F>(&mut self, rect: &D2D_RECT_F, draw_fn: F) -> Result<(), D2DBackendError>
    where
        F: FnOnce(&mut Self) -> Result<(), D2DBackendError>,
    {
        unsafe {
            // 1. 检查当前变换矩阵，判断是否有旋转
            let mut transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut transform);

            // 如果 M12 和 M21 都是 0，说明没有旋转和斜切 -> Fast Path
            // 浮点数比较用 epsilon
            let is_axis_aligned =
                transform.M12.abs() < f32::EPSILON && transform.M21.abs() < f32::EPSILON;

            let mut active_layer: Option<ID2D1Layer> = None;

            if is_axis_aligned {
                // === Fast Path: 轴对齐裁剪 ===
                // 这种方式利用硬件 Scissor Test，极快，但不抗锯齿，且只能切正矩形
                self.current_render
                    .PushAxisAlignedClip(rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            } else {
                // === Quality Path: 图层几何掩码 ===
                // 有旋转，必须用 Layer Mask，否则切出来的框是错的

                // A. 获取复用的 Layer
                let layer = self.get_layer()?;

                // B. 创建裁剪几何体 (这一步也可以加 LRU 缓存，参考之前的 path cache，这里先简单做)
                // 注意：CreateRectangleGeometry 开销非常小
                let clip_geometry = RenderFactory::get().factory.CreateRectangleGeometry(rect)?;

                // C. 配置 Layer 参数
                let layer_params = D2D1_LAYER_PARAMETERS1 {
                    contentBounds: D2D_RECT_F {
                        left: -f32::INFINITY,
                        top: -f32::INFINITY,
                        right: f32::INFINITY,
                        bottom: f32::INFINITY,
                    },
                    geometricMask: ManuallyDrop::new(Some(clip_geometry.cast()?)), // 绑定掩码
                    maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                    maskTransform: Matrix3x2::identity(), // 掩码跟随当前变换
                    opacity: 1.0,
                    opacityBrush: ManuallyDrop::new(None),
                    layerOptions: D2D1_LAYER_OPTIONS1_NONE,
                };

                self.current_render.PushLayer(&layer_params, &layer);
                active_layer = Some(layer);
            }

            // 2. 执行绘制
            let result = draw_fn(self);

            // 3. 恢复现场
            if is_axis_aligned {
                self.current_render.PopAxisAlignedClip();
            } else {
                self.current_render.PopLayer();
                // 归还 Layer 到池子
                if let Some(layer) = active_layer {
                    self.return_layer(layer);
                }
            }

            result
        }
    }
}

// 辅助函数：获取 SVG 的固有尺寸 (Intrinsic Size)
#[cfg(feature = "svg")]
fn get_svg_size(doc: &ID2D1SvgDocument) -> Result<(f32, f32), D2DBackendError> {
    unsafe {
        let root = doc.GetRoot()?;

        let mut view_box = D2D1_SVG_VIEWBOX::default();

        if root
            .GetAttributeValue2(
                w!("viewBox"),
                D2D1_SVG_ATTRIBUTE_POD_TYPE_VIEWBOX,
                &mut view_box as *mut _ as *mut c_void,
                size_of::<D2D1_SVG_VIEWBOX>() as u32,
            )
            .is_ok()
        {
            return Ok((view_box.width, view_box.height));
        }

        let mut w_ok = 0f32;
        let mut h_ok = 0f32;
        root.GetAttributeValue2(
            w!("width"),
            D2D1_SVG_ATTRIBUTE_POD_TYPE_FLOAT,
            &mut w_ok as *mut _ as *mut c_void,
            size_of::<D2D1_SVG_VIEWBOX>() as u32,
        )?;
        root.GetAttributeValue2(
            w!("height"),
            D2D1_SVG_ATTRIBUTE_POD_TYPE_FLOAT,
            &mut h_ok as *mut _ as *mut c_void,
            size_of::<f32>() as u32,
        )?;

        Ok((w_ok, h_ok))
    }
}

// 计算 Quad 形状的 Hash (不包含位置信息 left/top)
fn compute_quad_hash(width: f32, height: f32, radius: f32) -> u64 {
    let mut s = DefaultHasher::new();
    // 乘 100 转整以保留精度，同时作为 Key
    ((width * 100.0) as i32).hash(&mut s);
    ((height * 100.0) as i32).hash(&mut s);
    ((radius * 100.0) as i32).hash(&mut s);
    // 这是一个 Magic Number，防止和其他形状的 Hash 碰撞
    10086.hash(&mut s);
    s.finish()
}

// 放在 impl 外面或 util 模块
fn compute_stroke_rect_hash(width: f32, height: f32, border_width: f32) -> u64 {
    let mut s = DefaultHasher::new();
    ((width * 100.0) as i32).hash(&mut s);
    ((height * 100.0) as i32).hash(&mut s);
    ((border_width * 100.0) as i32).hash(&mut s);
    // 使用不同的 Magic Number (比如 10010) 区分于 fill_quad
    "STROKE_RECT".hash(&mut s);
    s.finish()
}

// 计算 Path 的 Hash Key
// 区分 Fill 和 Draw (描边)，因为描边还受 stroke_width 影响
fn compute_path_cache_key(path: &Path, stroke_width: f32, is_stroke: bool) -> u64 {
    let mut s = DefaultHasher::new();
    path.hash(&mut s); // 假设 Path 实现了 Hash
    if is_stroke {
        ((stroke_width * 100.0) as i32).hash(&mut s);
        "PATH_STROKE".hash(&mut s);
    } else {
        "PATH_FILL".hash(&mut s);
    }
    s.finish()
}

/// De Casteljau 算法计算贝塞尔曲线上的点
/// points: 包含起点的所有控制点
fn calculate_bezier_point(t: f32, points: &[Vector2]) -> Vector2 {
    if points.is_empty() {
        return Vector2 { X: 0.0, Y: 0.0 };
    }
    if points.len() == 1 {
        return points[0];
    }

    // 我们可以复用 buffer 避免每次递归都 alloc，但简单的递归/循环更容易理解
    // 这里使用迭代版 De Casteljau
    let mut temp = points.to_vec();
    let n = temp.len();

    // 每一层减少一个点
    for k in 1..n {
        for i in 0..(n - k) {
            // Linear Interpolation: P_i = (1-t)*P_i + t*P_{i+1}
            let p0 = temp[i];
            let p1 = temp[i + 1];
            temp[i] = Vector2 {
                X: (1.0 - t) * p0.X + t * p1.X,
                Y: (1.0 - t) * p0.Y + t * p1.Y,
            };
        }
    }

    temp[0]
}
