use flor_base::graphics::SvgHandle;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::Graphics::Direct2D::{ID2D1CommandList, ID2D1Effect, ID2D1SvgDocument};

#[derive(Debug)]
pub(crate) struct SvgShadowCache {
    pub command_list: ID2D1CommandList,
    pub shadow_effect: ID2D1Effect,
    pub last_blur_radius: f32,
}

#[derive(Debug, Clone)]
pub struct D2DSvgHandle {
    raw: ID2D1SvgDocument,
    pub(crate) shadow_cache: Arc<Mutex<Option<SvgShadowCache>>>,
}

impl SvgHandle for D2DSvgHandle {}

impl D2DSvgHandle {
    pub fn new(raw: ID2D1SvgDocument) -> Self {
        Self {
            raw,
            shadow_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn raw(&self) -> &ID2D1SvgDocument {
        &self.raw
    }
}
