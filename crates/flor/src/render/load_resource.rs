#[cfg(feature = "svg")]
use crate::render::FlorSvgHandle;
use crate::render::{FlorImageHandle, FlorRendererError};

pub trait LoadRenderResource {
    fn load_image(&self, image: &[u8]) -> Result<FlorImageHandle, FlorRendererError>;
    fn load_raw_image(
        &self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<FlorImageHandle, FlorRendererError>;
    #[cfg(feature = "svg")]
    fn load_svg(&self, svg: &[u8]) -> Result<FlorSvgHandle, FlorRendererError>;
}
