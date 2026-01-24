use flor_base::types::Color;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

pub trait AsD2dColor {
    fn as_d2d_color(&self) -> D2D1_COLOR_F;
}

impl AsD2dColor for Color {
    fn as_d2d_color(&self) -> D2D1_COLOR_F {
        D2D1_COLOR_F {
            r: self.r as f32 / 255.0,
            g: self.g as f32 / 255.0,
            b: self.b as f32 / 255.0,
            a: self.a as f32 / 255.0,
        }
    }
}
