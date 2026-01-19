use flor_base::graphics::SvgHandle;
use graphics::handle::D2DSvgHandle;

#[derive(Debug, Clone)]
pub enum FlorSvgHandle {
    D2DSvgHandle(D2DSvgHandle),
}

impl SvgHandle for FlorSvgHandle {}
