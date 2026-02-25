#[cfg(feature = "tiny-skia")]
use crate::graphics_cpu::handle::TinySkiaImageHandle;
#[cfg(feature = "direct2d")]
use crate::graphics_gpu::handle::D2DImageHandle;
#[cfg(feature = "opengl")]
use crate::graphics_gpu::handle::GlImageHandle;
use flor_base::graphics::ImageHandle;

#[derive(Debug, Clone)]
pub enum FlorImageHandle {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DImageHandle,
        #[cfg(feature = "opengl")] GlImageHandle,
    ),
    #[cfg(feature = "cpu-render-backend")]
    CPU(#[cfg(feature = "tiny-skia")] TinySkiaImageHandle),
}

impl ImageHandle for FlorImageHandle {
    fn frame_count(&self) -> usize {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.frame_count(),
            #[cfg(feature = "cpu-render-backend")]
            FlorImageHandle::CPU(handle) => handle.frame_count(),
        }
    }

    fn delays(&self) -> &[u16] {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.delays(),
            #[cfg(feature = "cpu-render-backend")]
            FlorImageHandle::CPU(handle) => handle.delays(),
        }
    }

    fn total_delays(&self) -> u128 {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.total_delays(),
            #[cfg(feature = "cpu-render-backend")]
            FlorImageHandle::CPU(handle) => handle.total_delays(),
        }
    }

    fn get_size(&self) -> (u32, u32) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.get_size(),
            #[cfg(feature = "cpu-render-backend")]
            FlorImageHandle::CPU(handle) => handle.get_size(),
        }
    }

    fn get_width(&self) -> u32 {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.get_width(),
            #[cfg(feature = "cpu-render-backend")]
            FlorImageHandle::CPU(handle) => handle.get_width(),
        }
    }

    fn get_height(&self) -> u32 {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.get_height(),
            #[cfg(feature = "cpu-render-backend")]
            FlorImageHandle::CPU(handle) => handle.get_height(),
        }
    }
}
