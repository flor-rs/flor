use crate::cursor::icon::CursorIcon;
use std::fmt::Debug;

pub trait CursorHandle: Debug + Clone + Send + Sync + 'static {
    type Handle;
    type Error;
    fn load_from_system(cursor_icon: CursorIcon) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn from_file_path(path: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn from_rgba_bytes(
        pixels: &[u8],
        width: u32,
        height: u32,
        hot_x: u32,
        hot_y: u32,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn handle(&self) -> Self::Handle;
}
