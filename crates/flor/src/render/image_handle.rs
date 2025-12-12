use flor_graphics_base::ImageHandle;
use graphics::handle::D2DImageHandle;

#[derive(Debug, Clone)]
pub enum FlorImageHandle {
    #[cfg(feature = "direct2d")]
    D2DImageHandle(D2DImageHandle),
}

impl ImageHandle for FlorImageHandle {
    fn frame_count(&self) -> usize {
        match self {
            #[cfg(feature = "direct2d")]
            FlorImageHandle::D2DImageHandle(handle) => handle.frame_count(),
        }
    }

    fn delays(&self) -> &[u16] {
        match self {
            #[cfg(feature = "direct2d")]
            FlorImageHandle::D2DImageHandle(handle) => handle.delays(),
        }
    }

    fn total_delays(&self) -> u128 {
        match self {
            #[cfg(feature = "direct2d")]
            FlorImageHandle::D2DImageHandle(handle) => handle.total_delays(),
        }
    }

    fn get_size(&self) -> (u32, u32) {
        match self {
            #[cfg(feature = "direct2d")]
            FlorImageHandle::D2DImageHandle(handle) => handle.get_size(),
        }
    }

    fn get_width(&self) -> u32 {
        match self {
            #[cfg(feature = "direct2d")]
            FlorImageHandle::D2DImageHandle(handle) => handle.get_width(),
        }
    }

    fn get_height(&self) -> u32 {
        match self {
            #[cfg(feature = "direct2d")]
            FlorImageHandle::D2DImageHandle(handle) => handle.get_height(),
        }
    }
}
