use crate::encode::encode_unicode;
use crate::error::D2DError;
use crate::handle::{D2DBrushHandle, D2DImageHandle, D2DSurfaceId, D2DTextFormatHandle};
use crate::render_factory::RenderFactory;
use crate::renderer::clip_type::ClipType;
use crate::text_layout::D2DTextLayout;
use crate::to_d2d::{AsD2dColor, IntoD2DTransform};
use flor_base::graphics::{
    Error, Gradient, HitTestResult, ImageDrawOptions, LayoutText, ParagraphAlignment, Path,
    PathCommand, PathDrawOptions, Render, RenderContext, ScaleMode, SurfaceDrawOptions,
    TextAlignment, TextChunk, TextDrawOptions, TextFormatHandle, TextTrimming, WordWrapping,
};
use flor_base::types::{Color, Rect, Transform2D};
use log::debug;
use lru::LruCache;
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::{fmt, slice};
use windows::core::{w, Interface, BOOL, HSTRING, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_BEZIER_SEGMENT, D2D1_BORDER_MODE, D2D1_BORDER_MODE_HARD, D2D1_COLOR_F,
    D2D1_COMPOSITE_MODE_SOURCE_OVER, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED,
    D2D1_FIGURE_END_OPEN, D2D1_GRADIENT_STOP, D2D_POINT_2U, D2D_RECT_F, D2D_RECT_U, D2D_SIZE_F,
    D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    CLSID_D2D1Crop, CLSID_D2D1GaussianBlur, CLSID_D2D1Shadow, ID2D1Bitmap1, ID2D1Brush,
    ID2D1CommandList, ID2D1DeviceContext, ID2D1DeviceContext5, ID2D1Effect, ID2D1Geometry,
    ID2D1HwndRenderTarget, ID2D1Image, ID2D1ImageBrush, ID2D1Layer, ID2D1PathGeometry1,
    ID2D1SolidColorBrush, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE, D2D1_BITMAP_OPTIONS_TARGET,
    D2D1_BITMAP_PROPERTIES1, D2D1_BRUSH_PROPERTIES, D2D1_BUFFER_PRECISION_8BPC_UNORM,
    D2D1_COLOR_INTERPOLATION_MODE_PREMULTIPLIED, D2D1_COLOR_SPACE_SRGB,
    D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_NONE, D2D1_CROP_PROP_RECT, D2D1_DRAW_TEXT_OPTIONS_NONE,
    D2D1_EXTEND_MODE_CLAMP, D2D1_GAUSSIANBLUR_PROP_BORDER_MODE,
    D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION, D2D1_IMAGE_BRUSH_PROPERTIES,
    D2D1_INTERPOLATION_MODE_LINEAR, D2D1_LAYER_OPTIONS1_NONE, D2D1_LAYER_PARAMETERS1,
    D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, D2D1_MAP_OPTIONS_READ, D2D1_PROPERTY_TYPE_ENUM,
    D2D1_PROPERTY_TYPE_FLOAT, D2D1_PROPERTY_TYPE_VECTOR4, D2D1_QUADRATIC_BEZIER_SEGMENT,
    D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES, D2D1_ROUNDED_RECT,
    D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION, D2D1_SHADOW_PROP_COLOR,
};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteTextLayout, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_HIT_TEST_METRICS, DWRITE_LINE_METRICS,
    DWRITE_LINE_SPACING_METHOD_DEFAULT, DWRITE_LINE_SPACING_METHOD_UNIFORM,
    DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
    DWRITE_PARAGRAPH_ALIGNMENT_FAR, DWRITE_PARAGRAPH_ALIGNMENT_NEAR, DWRITE_TEXT_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_JUSTIFIED, DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING,
    DWRITE_TEXT_METRICS, DWRITE_TEXT_RANGE, DWRITE_TRIMMING, DWRITE_TRIMMING_GRANULARITY_CHARACTER,
    DWRITE_TRIMMING_GRANULARITY_NONE, DWRITE_TRIMMING_GRANULARITY_WORD,
    DWRITE_WORD_WRAPPING_CHARACTER, DWRITE_WORD_WRAPPING_NO_WRAP, DWRITE_WORD_WRAPPING_WRAP,
};
use windows_numerics::{Matrix3x2, Vector2};

#[cfg(feature = "svg")]
use crate::handle::{D2DSvgHandle, SvgShadowCache};
#[cfg(feature = "svg")]
use flor_base::graphics::SvgDrawOptions;
#[cfg(feature = "svg")]
use std::ffi::c_void;
use std::hash::{DefaultHasher, Hash, Hasher};
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

#[cfg(feature = "memory-font")]
use crate::memory_font::{get_family_name_from_face, MemoryFontFileLoader};
#[cfg(feature = "memory-font")]
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFactory5, IDWriteFontFileLoader, DWRITE_FONT_FACE_TYPE_UNKNOWN,
    DWRITE_FONT_SIMULATIONS_NONE,
};

mod clip_type;
mod config;

pub use config::*;

pub struct D2DRenderer {
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
    pub clip_stack: Vec<ClipType>,
    pub transform_stack: Vec<Matrix3x2>,
    /// 暂停剪裁时的深度栈（支持嵌套暂停）
    pub suspended_clip_depths: Vec<usize>,
}

impl Debug for D2DRenderer {
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

impl Render for D2DRenderer {
    type HWND = HWND;
    type Render = Self;
    type Config = D2DConfig;

    fn create(
        hwnd: impl Into<Self::HWND>,
        width: u32,
        height: u32,
        wait_v_sync: bool,
        _config: Self::Config,
    ) -> Result<Self::Render, Self::Error> {
        debug!("create window render.");
        RenderFactory::try_init()?;
        Ok(RenderFactory::get().create_render(hwnd.into(), width, height, wait_v_sync)?)
    }
}

impl RenderContext for D2DRenderer {
    type Error = D2DError;
    type ImageHandle = D2DImageHandle;
    type SurfaceId = D2DSurfaceId;
    type BrushHandle = D2DBrushHandle;
    #[cfg(feature = "svg")]
    type SvgHandle = D2DSvgHandle;
    type TextFormatHandle = D2DTextFormatHandle;
    type LayoutText = D2DTextLayout;

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
        raw_bytes: &Vec<Vec<u8>>,
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

    #[cfg(feature = "memory-font")]
    fn create_text_format_from_bytes(
        &mut self,
        font_data: &[u8],
        ttc_index: u32,
    ) -> Result<Self::TextFormatHandle, Self::Error> {
        unsafe {
            let factory = &RenderFactory::get().write_factory;

            // 1. Loader 准备
            let loader_impl = MemoryFontFileLoader {
                data: font_data.to_vec(),
            };
            let loader_interface: IDWriteFontFileLoader = loader_impl.into();
            factory.RegisterFontFileLoader(&loader_interface)?;
            // 2. 创建 FontFile
            let dummy_key = 0u32;
            let font_file = factory.CreateCustomFontFileReference(
                &dummy_key as *const _ as *const _,
                4,
                &loader_interface,
            )?;

            // 3. 构建 Collection (Win10+ 必须步骤，否则 CreateTextFormat 找不到字体)
            // 需要 cast 到 Factory5
            let factory5: IDWriteFactory5 = factory.cast()?;
            let font_set_builder = factory5.CreateFontSetBuilder()?;
            font_set_builder.AddFontFile(&font_file)?;
            let font_set = font_set_builder.CreateFontSet()?;
            let font_collection = factory5.CreateFontCollectionFromFontSet(&font_set)?;

            // 4. 创建 Face 并解析名称
            // 因为 IDWriteFontFace 没有 GetFontFamily，我们需要手动解析 'name' 表
            let font_face = factory.CreateFontFace(
                DWRITE_FONT_FACE_TYPE_UNKNOWN,
                &[Some(font_file)], // 传递切片
                ttc_index,
                DWRITE_FONT_SIMULATIONS_NONE,
            )?;

            // 调用上面的辅助函数获取名称
            let family_name_string = get_family_name_from_face(&font_face)?;

            // 转换为 HSTRING / PCWSTR
            let family_name_hstring = HSTRING::from(&family_name_string);

            // 5. 创建 Format
            let text_format = factory.CreateTextFormat(
                PCWSTR(family_name_hstring.as_ptr()),
                &font_collection,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                16.0,
                w!("zh-cn"),
            )?;
            factory.UnregisterFontFileLoader(&loader_interface)?;
            Ok(D2DTextFormatHandle::new(text_format, family_name_string))
        }
    }

    fn create_text_layout(
        &self,
        text: String,
        bounds: Rect<f32>,
        default_text_format: Self::TextFormatHandle,
    ) -> Result<Self::LayoutText, Self::Error> {
        Ok(D2DTextLayout::create_text_layout(
            text,
            bounds,
            default_text_format,
        ))
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
        gradient: &Gradient,
    ) -> Result<Self::BrushHandle, Self::Error> {
        unsafe {
            let brush: ID2D1Brush = match &gradient {
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
                                radiusX: *radius,
                                radiusY: *radius,
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
                return Err(D2DError::from(Error::ImageFrameNotFound(frame_index)));
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
                        &(shadow_opts.blur_radius * 0.4).to_ne_bytes(),
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
            let svg_doc = handle.raw();
            let dc5: ID2D1DeviceContext5 = self.current_render.cast()?;

            let (src_w, src_h) = get_svg_size(svg_doc).unwrap_or((100.0, 100.0));

            let target_w = width.unwrap_or(src_w);
            let target_h = height.unwrap_or(src_h);

            // 解析 Options，新增 global_opacity 的获取
            let (scale_mode, user_transform, shadow, global_opacity) = match options {
                Some(opt) => (
                    opt.scale_mode.unwrap_or(ScaleMode::None),
                    opt.transform,
                    opt.shadow,
                    opt.opacity.unwrap_or(1.0),
                ),
                None => (ScaleMode::None, None, None, 1.0),
            };

            let (scale_x, scale_y, offset_x, offset_y) = match scale_mode {
                ScaleMode::None => (1.0, 1.0, x, y),
                ScaleMode::Stretch => (target_w / src_w, target_h / src_h, x, y),
                ScaleMode::Fit => {
                    let ratio = (target_w / src_w).min(target_h / src_h);
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
                _ => (1.0, 1.0, x, y),
            };

            let layout_matrix =
                Matrix3x2::scale(scale_x, scale_y) * Matrix3x2::translation(offset_x, offset_y);

            let user_matrix = if let Some(t) = user_transform {
                t.into_transform()
            } else {
                Matrix3x2::identity()
            };

            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            let final_transform = layout_matrix * user_matrix * old_transform;

            let need_clip = matches!(scale_mode, ScaleMode::Cover);
            if need_clip {
                let clip_rect = D2D_RECT_F {
                    left: x,
                    top: y,
                    right: x + target_w,
                    bottom: y + target_h,
                };
                self.current_render
                    .PushAxisAlignedClip(&clip_rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            }

            self.current_render.SetTransform(&final_transform);

            // ================= 新增：处理透明度图层 =================
            let is_transparent = global_opacity < 0.999;
            let mut active_layer: Option<ID2D1Layer> = None;

            if is_transparent {
                let layer = self.get_layer()?;
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
                    opacity: global_opacity,
                    opacityBrush: ManuallyDrop::new(None),
                    layerOptions: D2D1_LAYER_OPTIONS1_NONE,
                };
                self.current_render.PushLayer(&layer_params, &layer);
                active_layer = Some(layer);
            }
            // ========================================================

            match shadow {
                Some(shadow_opts) => {
                    let mut cache_access = handle.shadow_cache.lock();

                    if cache_access.is_none() {
                        let command_list = dc5.CreateCommandList()?;
                        let old_target = dc5.GetTarget().ok();

                        dc5.SetTarget(&command_list);
                        dc5.SetTransform(&Matrix3x2::identity());
                        dc5.DrawSvgDocument(handle.raw());
                        command_list.Close()?;
                        dc5.SetTarget(old_target.as_ref());

                        let shadow_effect = self.current_render.CreateEffect(&CLSID_D2D1Shadow)?;
                        shadow_effect.SetInput(0, &command_list, true);

                        *cache_access = Some(SvgShadowCache {
                            command_list,
                            shadow_effect,
                            last_blur_radius: -1.0,
                        });
                    }

                    let Some(cache) = cache_access.as_mut() else {
                        unreachable!();
                    };

                    if (cache.last_blur_radius - shadow_opts.blur_radius).abs() > f32::EPSILON {
                        cache.shadow_effect.SetValue(
                            D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                            D2D1_PROPERTY_TYPE_FLOAT,
                            &(shadow_opts.blur_radius * 0.4).to_ne_bytes(),
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

                    dc5.DrawImage(
                        &cache.command_list.cast::<ID2D1Image>()?,
                        None,
                        None,
                        D2D1_INTERPOLATION_MODE_LINEAR,
                        D2D1_COMPOSITE_MODE_SOURCE_OVER,
                    );
                }
                None => {
                    dc5.DrawSvgDocument(svg_doc);
                }
            }

            // ================= 新增：恢复透明度图层 =================
            if let Some(layer) = active_layer {
                self.current_render.PopLayer();
                self.return_layer(layer);
            }
            // ========================================================

            if need_clip {
                self.current_render.PopAxisAlignedClip();
            }
            self.current_render.SetTransform(&old_transform);

            Ok(())
        }
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
        unsafe {
            let bitmap = handle.raw().GetBitmap()?;

            let (scale_mode, transform, global_opacity, shadow) = match options {
                None => (ScaleMode::None, None, 1.0, None),
                Some(opt) => (
                    opt.scale_mode.unwrap_or(ScaleMode::None),
                    opt.transform,
                    opt.opacity.unwrap_or(1.0),
                    opt.shadow,
                ),
            };

            let size = bitmap.GetSize();
            let target_w = width.unwrap_or(size.width);
            let target_h = height.unwrap_or(size.height);

            // ---- 1. 布局计算 (Layout Calculation) ----
            let (mut final_w, mut final_h) = (size.width, size.height);
            let (mut offset_x, mut offset_y) = (0.0, 0.0);
            let mut need_clip = false;

            match scale_mode {
                ScaleMode::None => {}
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

            // ---- 2. 坐标系准备 ----
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            let user_matrix = if let Some(t) = transform {
                t.into_transform()
            } else {
                Matrix3x2::identity()
            };

            let scale_x = final_w / size.width;
            let scale_y = final_h / size.height;
            let layout_matrix = Matrix3x2::scale(scale_x, scale_y)
                * Matrix3x2::translation(x + offset_x, y + offset_y);

            let pre_clip_transform = user_matrix * old_transform;
            let final_transform = layout_matrix * pre_clip_transform;

            // ---- 3. 定义核心绘制逻辑 ----
            let draw_content = |render: &mut Self| -> Result<(), Self::Error> {
                render.current_render.SetTransform(&final_transform);

                let has_shadow = shadow.is_some();
                let is_transparent = global_opacity < 0.999;
                let use_transparency_layer = has_shadow && is_transparent;

                let mut active_layer: Option<ID2D1Layer> = None;

                if use_transparency_layer {
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
                        opacity: global_opacity,
                        opacityBrush: ManuallyDrop::new(None),
                        layerOptions: D2D1_LAYER_OPTIONS1_NONE,
                    };
                    render.current_render.PushLayer(&layer_params, &layer);
                    active_layer = Some(layer);
                }

                if let Some(shadow_opts) = shadow {
                    // Surface 动态更新，不进入 LRU 缓存，直接使用全局通用特效并强制更新数据 (invalidate=true)
                    let shadow_effect = render.get_or_create_shadow_effect()?;
                    shadow_effect.SetInput(0, &bitmap, true);

                    shadow_effect.SetValue(
                        D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                        D2D1_PROPERTY_TYPE_FLOAT,
                        &(shadow_opts.blur_radius * 0.4).to_ne_bytes(),
                    )?;

                    let d2d_color = shadow_opts.color.as_d2d_color();
                    shadow_effect.SetValue(
                        D2D1_SHADOW_PROP_COLOR.0 as u32,
                        D2D1_PROPERTY_TYPE_VECTOR4,
                        slice::from_raw_parts(&d2d_color as *const _ as *const u8, 16),
                    )?;

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
                    &bitmap,
                    Some(&local_rect),
                    draw_opacity,
                    D2D1_INTERPOLATION_MODE_LINEAR,
                    None,
                    None,
                );

                if let Some(layer) = active_layer {
                    render.current_render.PopLayer();
                    render.return_layer(layer);
                }

                Ok(())
            };

            // ---- 4. 执行绘制与裁剪 ----
            if need_clip {
                let clip_rect = D2D_RECT_F {
                    left: x,
                    top: y,
                    right: x + target_w,
                    bottom: y + target_h,
                };

                self.current_render.SetTransform(&pre_clip_transform);
                self.with_content_clip(&clip_rect, draw_content)?;
            } else {
                draw_content(self)?;
            }

            // ---- 5. 恢复矩阵 ----
            self.current_render.SetTransform(&old_transform);
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
        default_brush: &Self::BrushHandle,
        options: Option<&TextDrawOptions>,
    ) -> Result<(), Self::Error> {
        // 1. 创建文本布局（会自动确保 format 是最新的）
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

        // 2. 绘制文本布局
        self.draw_layout_text(&text_layout, default_brush, options)
    }

    fn draw_layout_text(
        &mut self,
        layout_text: &Self::LayoutText,
        default_brush: &Self::BrushHandle,
        options: Option<&TextDrawOptions>,
    ) -> Result<(), Self::Error> {
        // 1. 获取 DirectWrite 布局对象（会自动确保 format 是最新的）
        let dwrite_layout = layout_text.dwrite_layout()?;
        let bounds = layout_text.bounds();
        let text = layout_text.text();

        unsafe {
            // 3. 获取资源
            let d2d_format = layout_text.default_text_format().raw();
            let main_brush = default_brush.raw();

            // 4. 处理 矩阵变换 (Transform)
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
                left: bounds.x,
                top: bounds.y,
                right: bounds.x + bounds.w,
                bottom: bounds.y + bounds.h,
            };

            // 5. 处理 阴影 (Shadow)
            if let Some(opts) = options {
                if let Some(shadow) = opts.shadow {
                    let shadow_color = shadow.color.as_d2d_color();
                    let text_utf16 = encode_unicode(text);

                    // --- 分支 A: 真·模糊阴影 (Quality Path) ---
                    if shadow.blur_radius > 0.0 {
                        // 尝试获取 DC5 接口
                        if let Ok(dc5) = self.current_render.cast::<ID2D1DeviceContext5>() {
                            // 1. 录制文本形状
                            let command_list = dc5.CreateCommandList()?;
                            let old_target = dc5.GetTarget().ok();

                            dc5.SetTarget(&command_list);
                            // 录制时使用 Identity，位置由 DrawText 的 layout_rect 决定
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
                                &d2d_format,
                                &layout_rect,
                                &self.scratch_brush,
                                &svg_glyph_style,
                                0,
                                D2D1_DRAW_TEXT_OPTIONS_NONE,
                                DWRITE_MEASURING_MODE_NATURAL,
                            );

                            command_list.Close()?;
                            dc5.SetTarget(old_target.as_ref());

                            // 2. 绘制特效
                            dc5.SetTransform(&final_transform);

                            let shadow_effect = self.get_or_create_shadow_effect()?;

                            shadow_effect.SetInput(0, &command_list, true);
                            shadow_effect.SetValue(
                                D2D1_SHADOW_PROP_BLUR_STANDARD_DEVIATION.0 as u32,
                                D2D1_PROPERTY_TYPE_FLOAT,
                                &(shadow.blur_radius * 0.4).to_ne_bytes(),
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
                            &d2d_format,
                            &shadow_rect,
                            &self.scratch_brush,
                            D2D1_DRAW_TEXT_OPTIONS_NONE,
                            DWRITE_MEASURING_MODE_NATURAL,
                        );
                    }
                }
            }

            // 6. 绘制 主文本 (Main Text)
            // 使用 DrawTextLayout 绘制文本以支持不同颜色的文本块
            self.current_render.DrawTextLayout(
                Vector2 {
                    X: bounds.x,
                    Y: bounds.y,
                },
                &dwrite_layout,
                main_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
            );

            // 7. 恢复矩阵
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
                            &(shadow.blur_radius * 0.4).to_ne_bytes(),
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
                            &(shadow.blur_radius * 0.4).to_ne_bytes(),
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
                            &(shadow.blur_radius * 0.4).to_ne_bytes(),
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
                            &(shadow.blur_radius * 0.4).to_ne_bytes(),
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

    // =========================================================
    // 1. 普通矩形剪裁 (使用 PushAxisAlignedClip)
    // =========================================================
    // =========================================================
    // 1. 普通矩形剪裁 (智能选择 Fast Path 或 Quality Path)
    // =========================================================
    fn push_clip(&mut self, rect: (f32, f32, f32, f32)) -> Result<(), Self::Error> {
        unsafe {
            let mut transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut transform);

            // 检查当前是否有旋转或斜切 (m12 和 m21 是否接近 0)
            let is_axis_aligned =
                transform.M12.abs() < f32::EPSILON && transform.M21.abs() < f32::EPSILON;

            if is_axis_aligned {
                // [Fast Path] 无旋转 -> 使用硬件快速剪裁
                let d2d_rect = D2D_RECT_F {
                    left: rect.0,
                    top: rect.1,
                    right: rect.0 + rect.2,
                    bottom: rect.1 + rect.3,
                };
                self.current_render
                    .PushAxisAlignedClip(&d2d_rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
                self.clip_stack
                    .push(ClipType::AxisAligned { rect: d2d_rect });
            } else {
                // [Quality Path] 有旋转 -> 必须使用 Layer + Geometry Mask 才能切出正确的旋转后矩形
                let d2d_rect = D2D_RECT_F {
                    left: rect.0,
                    top: rect.1,
                    right: rect.0 + rect.2,
                    bottom: rect.1 + rect.3,
                };
                // 创建临时的矩形几何体 (CreateRectangleGeometry 开销非常小，无需缓存)
                let geometry = RenderFactory::get()
                    .factory
                    .CreateRectangleGeometry(&d2d_rect)?;

                // 复用现有的 push_layer 逻辑，获取 layer 和 params 以便保存
                let (layer, params) = self.push_layer_with_geometry(&geometry)?;

                // 保存完整数据以支持 suspend/resume
                self.clip_stack.push(ClipType::Layer { layer, params });
            }
        }

        Ok(())
    }

    // =========================================================
    // 2. 圆角剪裁 (使用 PushLayer + Geometry)
    // =========================================================
    fn push_rounded_clip(
        &mut self,
        rect: (f32, f32, f32, f32),
        radius: f32,
    ) -> Result<(), Self::Error> {
        unsafe {
            // A. 创建圆角矩形几何体
            let rounded_rect = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: rect.0,
                    top: rect.1,
                    right: rect.0 + rect.2,
                    bottom: rect.1 + rect.3,
                },
                radiusX: radius,
                radiusY: radius,
            };
            let geometry = RenderFactory::get()
                .factory
                .CreateRoundedRectangleGeometry(&rounded_rect)?;

            // B. 压入 Layer，获取 layer 和 params 以便保存
            let (layer, params) = self.push_layer_with_geometry(&geometry)?;

            // 保存完整数据以支持 suspend/resume
            self.clip_stack.push(ClipType::Layer { layer, params });
        }

        Ok(())
    }

    // =========================================================
    // 3. 路径剪裁 (使用 PushLayer + Geometry)
    // =========================================================
    fn push_path_clip(&mut self, path: &Path) -> Result<(), Self::Error> {
        // 注意：千万不要在这里调 PopAxisAlignedClip！
        // D2D 的剪裁是自动取交集的。如果你在这里 Pop，就破坏了父级的剪裁限制。

        unsafe {
            // A. 将你的 Path 转换为 D2D Geometry
            // 这里假设你有一个辅助方法 convert_path_to_d2d_geometry
            let geometry = self.build_geometry(path)?;

            // B. 压入 Layer，获取 layer 和 params 以便保存
            let (layer, params) = self.push_layer_with_geometry(&geometry)?;

            // 保存完整数据以支持 suspend/resume
            self.clip_stack.push(ClipType::Layer { layer, params });
        }

        Ok(())
    }

    fn pop_clip(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        let current_depth = self.clip_stack.len() as u32;
        let target = target_depth.unwrap_or_else(|| current_depth.saturating_sub(1));

        if target >= current_depth {
            return Ok(());
        }

        while self.clip_stack.len() as u32 > target {
            if let Some(clip_type) = self.clip_stack.pop() {
                unsafe {
                    match clip_type {
                        ClipType::AxisAligned { .. } => {
                            self.current_render.PopAxisAlignedClip();
                        }
                        ClipType::Layer { layer, .. } => {
                            self.current_render.PopLayer();
                            // 回收 layer 到 pool（限制最大 32 个）
                            if self.layer_pool.len() < 32 {
                                self.layer_pool.push(layer);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn get_clip_depth(&mut self) -> Result<u32, Self::Error> {
        Ok(self.clip_stack.len() as u32)
    }

    // =========================================================
    // 4. 暂停/恢复剪裁 (用于 Popup/Overlay 等需要逃逸父级剪裁的场景)
    // =========================================================

    fn suspend_clip(&mut self) -> Result<(), Self::Error> {
        let current_depth = self.clip_stack.len();
        let last_suspended_depth = self.suspended_clip_depths.last().copied().unwrap_or(0);

        // 记录当前剪裁栈深度
        self.suspended_clip_depths.push(current_depth);

        // 从后往前弹出属于当前活动层级的剪裁
        unsafe {
            for clip_type in self.clip_stack[last_suspended_depth..current_depth]
                .iter()
                .rev()
            {
                match clip_type {
                    ClipType::AxisAligned { .. } => {
                        self.current_render.PopAxisAlignedClip();
                    }
                    ClipType::Layer { .. } => {
                        self.current_render.PopLayer();
                    }
                }
            }
        }

        Ok(())
    }

    fn resume_clip(&mut self) -> Result<(), Self::Error> {
        // 如果没有暂停，不做任何事
        let Some(suspended_depth) = self.suspended_clip_depths.pop() else {
            return Ok(());
        };
        let last_suspended_depth = self.suspended_clip_depths.last().copied().unwrap_or(0);

        unsafe {
            // 1. 先 pop 掉本次暂停期间新增的所有剪裁（D2D 层面）
            //    这些是在 suspend 之后、resume 之前 push 的
            for clip_type in self.clip_stack[suspended_depth..].iter().rev() {
                match clip_type {
                    ClipType::AxisAligned { .. } => {
                        self.current_render.PopAxisAlignedClip();
                    }
                    ClipType::Layer { .. } => {
                        self.current_render.PopLayer();
                    }
                }
            }

            // 2. 回收本次暂停期间新增的 layer 到 pool，并截断 clip_stack
            while self.clip_stack.len() > suspended_depth {
                if let Some(clip_type) = self.clip_stack.pop() {
                    if let ClipType::Layer { layer, .. } = clip_type {
                        if self.layer_pool.len() < 32 {
                            self.layer_pool.push(layer);
                        }
                    }
                }
            }

            // 3. 重新 push 本次暂停前的剪裁（但只 push 属于当前活动层级的那一段）
            for clip_type in self.clip_stack[last_suspended_depth..suspended_depth].iter() {
                match clip_type {
                    ClipType::AxisAligned { rect } => {
                        self.current_render
                            .PushAxisAlignedClip(rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
                    }
                    ClipType::Layer { layer, params } => {
                        self.current_render.PushLayer(params, layer);
                    }
                }
            }
        }

        Ok(())
    }

    // =========================================================
    // 5. 变换
    // =========================================================

    fn push_transform(&mut self, transform: &Transform2D) -> Result<(), Self::Error> {
        let local = transform.into_transform();
        unsafe {
            let mut old_transform = Matrix3x2::default();
            self.current_render.GetTransform(&mut old_transform);

            // 压栈保存当前状态
            self.transform_stack.push(old_transform);

            // 计算并应用新变换 (Local * World)
            let new_transform = local * old_transform;
            self.current_render.SetTransform(&new_transform);
        }
        Ok(())
    }

    fn pop_transform(&mut self, target_depth: Option<u32>) -> Result<(), Self::Error> {
        let current_depth = self.transform_stack.len() as u32;
        let target = target_depth.unwrap_or_else(|| current_depth.saturating_sub(1));

        if target >= current_depth {
            return Ok(());
        }

        // 目标深度对应的 Transform 就是我们想要恢复的状态
        // transform_stack[i] 存储的是 "第 i+1 次 Push 之前的状态"
        // 当我们 restore 到 depth 时，说明我们只保留 depth 次 Push
        // 实际上 transform_stack 的长度就是 depth
        // 我们要恢复的是 stack[target] 吗？
        // 假设 stack = [M0, M1]。Len = 2。Current = M2.
        // target = 1. Want M1.
        // stack[1] == M1.
        // target = 0. Want M0.
        // stack[0] == M0.
        // 所以恢复 stack[target] 是正确的

        // 注意：Vec 索引 usize
        let target_idx = target as usize;
        let restore_transform = self.transform_stack[target_idx];

        unsafe {
            self.current_render.SetTransform(&restore_transform);
        }

        // 截断栈
        self.transform_stack.truncate(target_idx);

        Ok(())
    }

    fn get_transform_depth(&mut self) -> Result<u32, Self::Error> {
        Ok(self.transform_stack.len() as u32)
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

impl D2DRenderer {
    // 内部 helper：获取可复用的 Effect
    fn get_or_create_shadow_effect(&mut self) -> Result<ID2D1Effect, D2DError> {
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

    fn apply_text_chunks(
        &self,
        text_layout: &impl Interface,
        text: &str,
        chunks: Option<
            &[TextChunk<
                <D2DRenderer as RenderContext>::BrushHandle,
                <D2DRenderer as RenderContext>::TextFormatHandle,
            >],
        >,
    ) -> Result<(), D2DError> {
        // 辅助函数：将 UTF-8 索引转换为 UTF-16 索引
        let utf8_to_utf16 = |text: &str, utf8_idx: usize| -> usize {
            let mut utf16_idx = 0;
            let mut current_utf8 = 0;

            for c in text.chars() {
                if current_utf8 >= utf8_idx {
                    break;
                }
                utf16_idx += c.len_utf16();
                current_utf8 += c.len_utf8();
            }

            utf16_idx
        };

        unsafe {
            let text_layout = text_layout.cast::<IDWriteTextLayout>()?;
            if let Some(chunks) = chunks {
                for chunk in chunks {
                    let end = chunk.start + chunk.length;
                    if end <= text.len() {
                        let start_utf16 = utf8_to_utf16(text, chunk.start);
                        let end_utf16 = utf8_to_utf16(text, end);

                        let chunk_range = DWRITE_TEXT_RANGE {
                            startPosition: start_utf16 as u32,
                            length: (end_utf16 - start_utf16) as u32,
                        };

                        // 设置画笔
                        text_layout.SetDrawingEffect(chunk.brush.raw(), chunk_range)?;

                        // 直接使用 D2DTextFormatHandle 的方法获取属性
                        let (weight, style, stretch) = chunk.text_format.map_props();

                        // 设置字体家族
                        let family_name_utf16 = HSTRING::from(chunk.text_format.font_family_name());
                        text_layout.SetFontFamilyName(&family_name_utf16, chunk_range)?;

                        // 设置字体大小
                        text_layout.SetFontSize(chunk.text_format.font_size(), chunk_range)?;

                        // 设置字重、字体样式和拉伸
                        text_layout.SetFontWeight(weight, chunk_range)?;
                        text_layout.SetFontStyle(style, chunk_range)?;
                        text_layout.SetFontStretch(stretch, chunk_range)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 内部辅助：创建并压入 Layer
    ///
    /// 返回 (layer, params) 以便保存到 clip_stack 中支持 suspend/resume
    unsafe fn push_layer_with_geometry<G: Interface>(
        &mut self,
        geometry: &G,
    ) -> Result<(ID2D1Layer, D2D1_LAYER_PARAMETERS1), D2DError>
    where
        G: Into<ID2D1Geometry> + Clone,
    {
        // 1. 优先从 pool 取，否则创建新的 Layer
        let layer = self
            .layer_pool
            .pop()
            .map(Ok)
            .unwrap_or_else(|| self.current_render.CreateLayer(None))?;

        // 2. 配置 Layer 参数
        let params = D2D1_LAYER_PARAMETERS1 {
            contentBounds: D2D_RECT_F {
                left: -f32::INFINITY,
                top: -f32::INFINITY,
                right: f32::INFINITY,
                bottom: f32::INFINITY,
            },
            geometricMask: ManuallyDrop::new(Some(geometry.cast::<ID2D1Geometry>()?)),
            maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
            maskTransform: Matrix3x2::identity(),
            opacity: 1.0,
            opacityBrush: ManuallyDrop::new(None),
            layerOptions: D2D1_LAYER_OPTIONS1_NONE,
        };

        // 3. 压栈
        self.current_render.PushLayer(&params, &layer);

        Ok((layer, params))
    }

    /// 内部辅助：将 Path 转换为 Direct2D Geometry
    fn build_geometry(&self, path: &Path) -> Result<ID2D1PathGeometry1, D2DError> {
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
                        if !is_figure_started {
                            sink.BeginFigure(current_point, D2D1_FIGURE_BEGIN_FILLED);
                            is_figure_started = true;
                        }
                        let pt = Vector2 { X: *x, Y: *y };
                        sink.AddLine(pt);
                        current_point = pt;
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
    fn get_or_create_blur_effect(&mut self) -> Result<ID2D1Effect, D2DError> {
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
    fn get_scratch_bitmap(&mut self, width: u32, height: u32) -> Result<ID2D1Bitmap1, D2DError> {
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
                    bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
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
    fn get_or_create_crop_effect(&mut self) -> Result<ID2D1Effect, D2DError> {
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
    ) -> Result<(ID2D1ImageBrush, Vector2), D2DError> {
        unsafe {
            let bounds = geometry.GetBounds(Some(world_transform))?;

            let pad = (blur_radius * 3.0).ceil() as i32;

            // 转换为整数坐标 (向外取整，确保覆盖所有像素)
            let left_u = bounds.left.floor() as i32 - pad;
            let top_u = bounds.top.floor() as i32 - pad;
            let right_u = bounds.right.ceil() as i32 + pad;
            let bottom_u = bounds.bottom.ceil() as i32 + pad;

            let width_u = (right_u - left_u).max(1) as u32;
            let height_u = (bottom_u - top_u).max(1) as u32;

            // 1. 获取复用的储备 Bitmap
            let snapshot = self.get_scratch_bitmap(width_u, height_u)?;

            // 必须先清空 Snapshot 避免脏区域
            let old_target = self.current_render.GetTarget()?;
            self.current_render.SetTarget(&snapshot);
            let transparent = D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            };
            self.current_render.Clear(Some(&transparent));
            self.current_render.SetTarget(&old_target);

            let rt_size = self.current_render.GetPixelSize();
            let src_left = left_u.max(0);
            let src_top = top_u.max(0);
            let src_right = right_u.min(rt_size.width as i32);
            let src_bottom = bottom_u.min(rt_size.height as i32);

            let src_width = (src_right - src_left).max(0) as u32;
            let src_height = (src_bottom - src_top).max(0) as u32;

            // 如果部分在屏幕内，则复制屏幕内容到 Bitmap
            if src_width > 0 && src_height > 0 {
                let dest_point = D2D_POINT_2U {
                    x: (src_left - left_u) as u32,
                    y: (src_top - top_u) as u32,
                };
                let src_rect = D2D_RECT_U {
                    left: src_left as u32,
                    top: src_top as u32,
                    right: src_right as u32,
                    bottom: src_bottom as u32,
                };

                // D2D 严禁在存在 Clip / Layer 时调用 CopyFromRenderTarget，所以我们暂时挂起所有裁剪！
                self.suspend_clip()?;

                snapshot.CopyFromRenderTarget(
                    Some(&dest_point),
                    &self.current_render,
                    Some(&src_rect),
                )?;

                // 拷贝完成，恢复所有环境的裁剪
                self.resume_clip()?;
            }

            // 2. 使用 Crop Effect 裁剪出有效区域
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

            // 3. 模糊裁剪输出
            let blur_effect = self.get_or_create_blur_effect()?;
            let crop_output = crop_effect.GetOutput()?;
            blur_effect.SetInput(0, &crop_output, true);

            if (self.last_blur_radius - blur_radius).abs() > 0.001 {
                blur_effect.SetValue(
                    D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION.0 as u32,
                    D2D1_PROPERTY_TYPE_FLOAT,
                    &(blur_radius * 0.4).to_ne_bytes(),
                )?;
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
                sourceRectangle: crop_rect,
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
    fn get_layer(&mut self) -> Result<ID2D1Layer, D2DError> {
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
    fn with_content_clip<F>(&mut self, rect: &D2D_RECT_F, draw_fn: F) -> Result<(), D2DError>
    where
        F: FnOnce(&mut Self) -> Result<(), D2DError>,
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
fn get_svg_size(doc: &ID2D1SvgDocument) -> Result<(f32, f32), D2DError> {
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
