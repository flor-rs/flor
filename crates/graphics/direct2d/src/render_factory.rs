use crate::handle::D2DRenderer;
#[cfg(feature = "memory-font")]
use crate::memory_font::MemoryFontFileLoader;
use log::debug;
use lru::LruCache;
use std::mem::ManuallyDrop;
use std::num::NonZeroUsize;
use std::sync::OnceLock;
use windows::core::Interface;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1DeviceContext, ID2D1Factory2, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
    D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_PROPERTIES1, D2D1_FACTORY_TYPE_MULTI_THREADED,
    D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_PRESENT_OPTIONS_IMMEDIATELY, D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
    D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Imaging::{CLSID_WICImagingFactory, IWICImagingFactory};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

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

    #[cfg(feature = "memory-font")]
    pub memory_font_file_loader: MemoryFontFileLoader,
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
                    #[cfg(feature = "memory-font")]
                    memory_font_file_loader: MemoryFontFileLoader::default(),
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
    ) -> Result<D2DRenderer, windows::core::Error> {
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
            Ok(D2DRenderer {
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
                clip_stack: vec![],
                transform_stack: vec![],
                suspended_clip_depths: vec![],
            })
        }
    }
}
