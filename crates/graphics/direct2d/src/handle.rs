use flor_base::graphics::Error;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use windows::Win32::Graphics::Direct2D::{
    ID2D1Bitmap1, ID2D1BitmapRenderTarget, ID2D1Brush, ID2D1SvgDocument,
};
use windows::Win32::Graphics::DirectWrite::IDWriteTextFormat;

#[cfg(feature = "svg")]
mod svg;
#[cfg(feature = "svg")]
pub use svg::*;

mod brush;
mod image;
mod surface;
mod text_format;

pub use {brush::*, image::*, surface::*, text_format::*};

pub trait GetRef<T> {
    /// 获取 Brush 的引用。
    ///
    /// 返回值是一个 MappedMutexGuard，它持有锁，但表现得像 &mut ID2D1Brush。
    /// 只有当 Brush 离开作用域时，锁才会释放。
    fn get_mut_ref(&self, id: &usize) -> Result<T, Error>;
}

impl GetRef<ID2D1Brush> for Arc<Mutex<FxHashMap<usize, ID2D1Brush>>> {
    fn get_mut_ref(&self, id: &usize) -> Result<ID2D1Brush, Error> {
        let lock_map = self.lock();
        lock_map
            .get(id)
            .cloned()
            .ok_or(Error::BrushHandleNotFound(*id))
    }
}

impl GetRef<ID2D1Bitmap1> for Arc<Mutex<FxHashMap<usize, ID2D1Bitmap1>>> {
    fn get_mut_ref(&self, id: &usize) -> Result<ID2D1Bitmap1, Error> {
        let lock_map = self.lock();
        lock_map
            .get(id)
            .cloned()
            .ok_or(Error::ImageHandleNotFound(*id))
    }
}

impl GetRef<ID2D1BitmapRenderTarget> for Arc<Mutex<FxHashMap<usize, ID2D1BitmapRenderTarget>>> {
    fn get_mut_ref(&self, id: &usize) -> Result<ID2D1BitmapRenderTarget, Error> {
        let lock_map = self.lock();
        lock_map
            .get(id)
            .cloned()
            .ok_or(Error::SurfaceIdHandleNotFound(*id))
    }
}

impl GetRef<ID2D1SvgDocument> for Arc<Mutex<FxHashMap<usize, ID2D1SvgDocument>>> {
    fn get_mut_ref(&self, id: &usize) -> Result<ID2D1SvgDocument, Error> {
        let lock_map = self.lock();
        lock_map
            .get(id)
            .cloned()
            .ok_or(Error::SvgHandleNotFound(*id))
    }
}

impl GetRef<IDWriteTextFormat> for Arc<Mutex<FxHashMap<usize, IDWriteTextFormat>>> {
    fn get_mut_ref(&self, id: &usize) -> Result<IDWriteTextFormat, Error> {
        let lock_map = self.lock();
        lock_map
            .get(id)
            .cloned()
            .ok_or(Error::TextFormatHandleNotFound(*id))
    }
}
