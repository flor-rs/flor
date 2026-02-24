use flor_base::graphics::ImageHandle;
#[cfg(feature = "direct2d")]
use graphics::handle::D2DImageHandle;
#[cfg(feature = "opengl")]
use graphics::handle::GlImageHandle;

#[derive(Debug, Clone)]
pub enum FlorImageHandle {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DImageHandle,
        #[cfg(feature = "opengl")] GlImageHandle,
    ),
}

impl ImageHandle for FlorImageHandle {
    fn frame_count(&self) -> usize {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.frame_count(),
        }
    }

    fn delays(&self) -> &[u16] {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.delays(),
        }
    }

    fn total_delays(&self) -> u128 {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.total_delays(),
        }
    }

    fn get_size(&self) -> (u32, u32) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.get_size(),
        }
    }

    fn get_width(&self) -> u32 {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.get_width(),
        }
    }

    fn get_height(&self) -> u32 {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            FlorImageHandle::GPU(handle) => handle.get_height(),
        }
    }
}
