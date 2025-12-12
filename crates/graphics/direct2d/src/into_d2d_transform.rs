use windows_numerics::Matrix3x2;
use flor_graphics_base::Transform2D;

pub trait IntoD2DTransform {
    fn into_transform(self) -> Matrix3x2;
}

impl IntoD2DTransform for Transform2D {
    fn into_transform(self) -> Matrix3x2 {
        Matrix3x2 {
            M11: self.m11,
            M12: self.m12,
            M21: self.m21,
            M22: self.m22,
            M31: self.dx,
            M32: self.dy,
        }
    }
}