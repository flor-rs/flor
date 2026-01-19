use flor_base::platform::DropEffect;
use windows::Win32::System::Ole::{
    DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_MOVE, DROPEFFECT_NONE,
    DROPEFFECT_SCROLL,
};

pub trait ToWinDropEffect {
    fn to_win32(self) -> DROPEFFECT;
}

impl ToWinDropEffect for DropEffect {
    fn to_win32(self) -> DROPEFFECT {
        match self {
            DropEffect::Copy => DROPEFFECT_COPY,
            DropEffect::Link => DROPEFFECT_LINK,
            DropEffect::Move => DROPEFFECT_MOVE,
            DropEffect::None => DROPEFFECT_NONE,
            DropEffect::Scroll => DROPEFFECT_SCROLL,
        }
    }
}
