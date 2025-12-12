

// use std::ffi::c_void;
// use windows::core::Interface;
// use windows::Win32::Graphics::Direct2D::{ID2D1BitmapBrush, ID2D1Brush, ID2D1ImageBrush, ID2D1LinearGradientBrush, ID2D1RadialGradientBrush, ID2D1SolidColorBrush};
//
// pub struct Brush(pub *mut c_void);
//
// impl Brush {
//     pub fn new(brush: ID2D1SolidColorBrush) -> Self {
//         Brush(brush.into_raw())
//     }
//
//     pub fn borrow_id2d_brush(&self) -> Option<&ID2D1Brush> {
//         unsafe { ID2D1Brush::from_raw_borrowed(&self.0) }
//     }
//
//     pub fn borrow_id2d_brush_expect(&self) -> &ID2D1Brush {
//         unsafe { ID2D1Brush::from_raw_borrowed(&self.0).expect("brush borrowed fail.") }
//     }
// }
//
// impl Clone for Brush {
//     fn clone(&self) -> Self {
//         self.borrow_id2d_brush_expect().clone().into()
//     }
// }
//
// impl From<ID2D1Brush> for Brush {
//     fn from(value: ID2D1Brush) -> Brush {
//         Self(value.into_raw())
//     }
// }
//
// impl Into<ID2D1Brush> for Brush {
//     fn into(self) -> ID2D1Brush {
//         unsafe { ID2D1Brush::from_raw(self.0) }
//     }
// }
//
// impl From<ID2D1BitmapBrush> for Brush {
//     fn from(value: ID2D1BitmapBrush) -> Brush {
//         Self(value.into_raw())
//     }
// }
//
// impl From<ID2D1SolidColorBrush> for Brush {
//     fn from(value: ID2D1SolidColorBrush) -> Brush {
//         Self(value.into_raw())
//     }
// }
//
// impl From<ID2D1ImageBrush> for Brush {
//     fn from(value: ID2D1ImageBrush) -> Brush {
//         Self(value.into_raw())
//     }
// }
//
// impl From<ID2D1LinearGradientBrush> for Brush {
//     fn from(value: ID2D1LinearGradientBrush) -> Brush {
//         Self(value.into_raw())
//     }
// }
//
// impl From<ID2D1RadialGradientBrush> for Brush {
//     fn from(value: ID2D1RadialGradientBrush) -> Brush {
//         Self(value.into_raw())
//     }
// }
//
// impl Drop for Brush {
//     fn drop(&mut self) {
//         unsafe { ID2D1Brush::from_raw(self.0); }
//         print!("{:?}", self.0);
//     }
// }
