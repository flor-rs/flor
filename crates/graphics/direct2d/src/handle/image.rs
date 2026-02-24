mod frame_metadata;

use crate::error::D2DError;
use flor_base::graphics::ImageHandle;
use lru::LruCache;
use parking_lot::Mutex;
use std::mem::ManuallyDrop;
use std::num::NonZeroUsize;
use std::sync::Arc;
use windows::core::{Error, GUID};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_RECT_F, D2D_SIZE_F,
    D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    ID2D1Bitmap1, ID2D1DeviceContext, ID2D1Effect, D2D1_ANTIALIAS_MODE_ALIASED,
    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_PROPERTIES1,
    D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_NONE,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM,
};
use windows::Win32::Graphics::Imaging::{
    GUID_ContainerFormatGif, GUID_WICPixelFormat32bppPBGRA, IWICBitmapDecoder,
    WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnLoad,
};
pub use {crate::*, frame_metadata::*};

#[derive(Debug, Clone)]
pub struct D2DImageHandle {
    frame_count: usize,
    pub(crate) bitmaps: Vec<ID2D1Bitmap1>,
    pub(crate) frame_shadow_cache: Arc<Mutex<LruCache<usize, ID2D1Effect>>>,
    delays: Vec<u16>,
    width: u32,
    height: u32,
    total_delays: u128,
}

impl ImageHandle for D2DImageHandle {
    fn frame_count(&self) -> usize {
        self.frame_count
    }

    fn delays(&self) -> &[u16] {
        &self.delays
    }

    fn total_delays(&self) -> u128 {
        self.total_delays
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }
}

impl D2DImageHandle {
    pub fn new(
        bitmaps: Vec<ID2D1Bitmap1>,
        frame_count: usize,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Self {
        let total_delays = delays.iter().map(|&d| d as u128).sum();
        Self {
            frame_count,
            bitmaps,
            frame_shadow_cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(frame_count)
                    .unwrap_or(unsafe { NonZeroUsize::new_unchecked(32) }),
            ))),
            delays,
            width,
            height,
            total_delays,
        }
    }

    pub(crate) fn from_bytes<'a>(
        bytes: &[u8],
        current_render: &ID2D1DeviceContext,
        render_factory: &'a RenderFactory,
    ) -> Result<Self, Error> {
        unsafe {
            let wic_stream = render_factory.wic_factory.CreateStream()?;
            wic_stream.InitializeFromMemory(bytes)?;

            let decoder = render_factory.wic_factory.CreateDecoderFromStream(
                &wic_stream,
                &GUID::default(),
                WICDecodeMetadataCacheOnLoad,
            )?;

            let frame_count = decoder.GetFrameCount()?;

            // 1. 获取全局画布尺寸 (通常取第一帧尺寸，或者读取全局元数据)
            let first_frame = decoder.GetFrame(0)?;

            let mut global_width = 0;
            let mut global_height = 0;
            first_frame.GetSize(&mut global_width, &mut global_height)?;

            let container_format = decoder.GetContainerFormat()?;

            #[allow(non_upper_case_globals)]
            match container_format {
                GUID_ContainerFormatGif => Self::decode_dynamic_image(
                    current_render,
                    global_width,
                    global_height,
                    frame_count,
                    decoder,
                    render_factory,
                ),
                _ => Self::decode_static_image(
                    current_render,
                    global_width,
                    global_height,
                    decoder,
                    render_factory,
                ),
            }
        }
    }

    pub unsafe fn decode_dynamic_image(
        current_render: &ID2D1DeviceContext,
        global_width: u32,
        global_height: u32,
        frame_count: u32,
        decoder: IWICBitmapDecoder,
        render_factory: &RenderFactory,
    ) -> Result<Self, Error> {
        // 2. 创建离屏渲染目标 (画布)
        let canvas_rt = current_render.CreateCompatibleRenderTarget(
            Some(&D2D_SIZE_F {
                width: global_width as f32,
                height: global_height as f32,
            }),
            None,
            None,
            D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_NONE,
        )?;

        // 初始化画布：全透明
        canvas_rt.BeginDraw();
        canvas_rt.Clear(Some(&D2D1_COLOR_F {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        }));
        canvas_rt.EndDraw(None, None)?;

        let mut d2d_bitmaps = Vec::with_capacity(frame_count as usize);
        let mut delays = Vec::with_capacity(frame_count as usize);

        // 用于 "Restore to Previous" (Disposal 3)
        let mut backup_bitmap = None;
        let mut prev_meta = FrameMetadata::default();

        let rf = RenderFactory::get();

        for i in 0..frame_count {
            let frame = decoder.GetFrame(i)?;

            // A. 读取元数据
            let query_reader = frame.GetMetadataQueryReader()?;
            let mut current_meta = FrameMetadata::get_frame_metadata(&query_reader)?;
            delays.push(current_meta.delay);

            // 【重要优化】：宽和高直接从 WIC Frame 获取更靠谱
            // metadata 里的宽高有时候是裁剪后的，有时候是逻辑屏幕的，容易乱
            let mut fw = 0;
            let mut fh = 0;
            frame.GetSize(&mut fw, &mut fh)?;
            if current_meta.width == 0.0 {
                current_meta.width = fw as f32;
            }
            if current_meta.height == 0.0 {
                current_meta.height = fh as f32;
            }

            // B. 处理上一帧的 Disposal (在绘制当前帧之前，处理画布状态)
            canvas_rt.BeginDraw();
            match prev_meta.disposal {
                2 => {
                    // Restore to Background (清空上一帧区域)
                    canvas_rt.PushAxisAlignedClip(
                        &D2D_RECT_F {
                            left: prev_meta.left,
                            top: prev_meta.top,
                            right: prev_meta.left + prev_meta.width,
                            bottom: prev_meta.top + prev_meta.height,
                        },
                        D2D1_ANTIALIAS_MODE_ALIASED,
                    );
                    canvas_rt.Clear(Some(&D2D1_COLOR_F {
                        r: 0.,
                        g: 0.,
                        b: 0.,
                        a: 0.,
                    }));
                    canvas_rt.PopAxisAlignedClip();
                }
                3 => {
                    // Restore to Previous (恢复备份)
                    if let Some(backup) = &backup_bitmap {
                        canvas_rt.DrawBitmap(
                            backup,
                            None,
                            1.0,
                            D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                            None,
                        );
                    } else {
                        canvas_rt.Clear(Some(&D2D1_COLOR_F {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 0.,
                        }));
                    }
                }
                _ => {} // 0 or 1: Keep (保留，直接叠加)
            }

            // 如果当前帧要求 RestoreToPrevious，在绘制它之前，备份当前干净的画布
            if current_meta.disposal == 3 {
                // 注意：这里需要 Copy，不能直接持有引用
                let rt_bitmap = canvas_rt.GetBitmap()?;
                let size = rt_bitmap.GetSize();
                let backup = current_render.CreateBitmap(
                    D2D_SIZE_U {
                        width: size.width as u32,
                        height: size.height as u32,
                    },
                    None,
                    0,
                    &rf.d2d1_bitmap_properties1,
                )?;
                backup.CopyFromBitmap(None, &rt_bitmap, None)?;
                backup_bitmap = Some(backup);
            }

            // C. 绘制当前帧 (WIC -> D2D 叠加)
            let format_converter = render_factory.wic_factory.CreateFormatConverter()?;
            format_converter.Initialize(
                &frame,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )?;
            let frame_bitmap = canvas_rt.CreateBitmapFromWicBitmap(&format_converter, None)?;

            let dest_rect = D2D_RECT_F {
                left: current_meta.left,
                top: current_meta.top,
                right: current_meta.left + current_meta.width,
                bottom: current_meta.top + current_meta.height,
            };

            canvas_rt.DrawBitmap(
                &frame_bitmap,
                Some(&dest_rect),
                1.0,
                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                None,
            );
            canvas_rt.EndDraw(None, None)?;

            // ==========================================
            // D. 【关键修复】: 保存结果
            // ==========================================
            // 1. 获取 Canvas 的视图
            let rt_bitmap = canvas_rt.GetBitmap()?;
            let size = rt_bitmap.GetSize();

            // 2. 创建一个全新的 Bitmap (显存独立)
            // 这一步是必须的！否则 d2d_bitmaps 里的所有图片都会指向同一个不断变化的 render target
            let snapshot_bitmap = current_render.CreateBitmap(
                D2D_SIZE_U {
                    width: size.width as u32,
                    height: size.height as u32,
                },
                None,
                0,
                &D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 96.,
                    dpiY: 96.,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                    colorContext: ManuallyDrop::new(None),
                },
            )?;

            // 3. 将当前 Canvas 的内容 Copy 到新 Bitmap 中
            snapshot_bitmap.CopyFromBitmap(None, &rt_bitmap, None)?;

            d2d_bitmaps.push(snapshot_bitmap);

            // 更新 loop 变量
            prev_meta = current_meta;
        }
        Ok(Self::new(
            d2d_bitmaps,
            frame_count as usize,
            global_width,
            global_height,
            delays,
        ))
    }

    pub unsafe fn decode_static_image(
        current_render: &ID2D1DeviceContext,
        global_width: u32,
        global_height: u32,
        decoder: IWICBitmapDecoder,
        render_factory: &RenderFactory,
    ) -> Result<Self, Error> {
        let frame = decoder.GetFrame(0)?;
        let format_converter = render_factory.wic_factory.CreateFormatConverter()?;
        format_converter.Initialize(
            &frame,
            &GUID_WICPixelFormat32bppPBGRA,
            WICBitmapDitherTypeNone,
            None,
            0.0,
            WICBitmapPaletteTypeCustom,
        )?;
        let frame_bitmap = current_render.CreateBitmapFromWicBitmap(&format_converter, None)?;

        Ok(Self::new(
            vec![frame_bitmap],
            0,
            global_width,
            global_height,
            vec![0],
        ))
    }

    pub(crate) fn from_raw_bytes(
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
        current_render: &ID2D1DeviceContext,
    ) -> Result<Self, D2DError> {
        if raw_bytes.len() != delays.len() {
            return Err(D2DError::RendererBaseError(
                flor_base::graphics::Error::ImageFrameDelayMismatch(raw_bytes.len(), delays.len()),
            ));
        }

        let frame_count = raw_bytes.len();
        let stride = width * 4; // 32位 RGBA/BGRA

        // 注意：image-rs 输出通常为 R8G8B8A8，D2D 默认偏好 Premultiplied Alpha
        let bitmap_props = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_R8G8B8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
            colorContext: ManuallyDrop::new(None),
        };

        let mut bitmaps = Vec::with_capacity(frame_count);

        for frame_data in raw_bytes {
            let bitmap = unsafe {
                current_render.CreateBitmap(
                    D2D_SIZE_U { width, height },
                    Some(frame_data.as_ptr() as _),
                    stride,
                    &bitmap_props,
                )?
            };
            bitmaps.push(bitmap);
        }
        Ok(Self::new(bitmaps, frame_count, width, height, delays))
    }
}
