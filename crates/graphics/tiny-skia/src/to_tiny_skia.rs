use flor_base::types::{Color, Transform2D};

pub trait ToTinySkia<T> {
    fn to_tiny_skia(&self) -> T;
}

impl ToTinySkia<tiny_skia::Color> for Color {
    fn to_tiny_skia(&self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(self.r, self.g, self.b, self.a)
    }
}

impl ToTinySkia<tiny_skia::Transform> for Transform2D {
    fn to_tiny_skia(&self) -> tiny_skia::Transform {
        tiny_skia::Transform::from_row(self.m11, self.m12, self.m21, self.m22, self.dx, self.dy)
    }
}
