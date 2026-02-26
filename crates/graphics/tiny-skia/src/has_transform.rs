use flor_base::graphics::{ImageDrawOptions, PathDrawOptions, TextDrawOptions};
use flor_base::types::Transform2D;

pub trait HasTransform {
    fn get_transform(&self) -> Option<&Transform2D>; // 假设你的变换类型是 Transform2D
}

// 为 ImageDrawOptions 实现特征
impl HasTransform for ImageDrawOptions {
    fn get_transform(&self) -> Option<&Transform2D> {
        self.transform.as_ref()
    }
}

// 为 PathDrawOptions 实现特征
impl HasTransform for PathDrawOptions {
    fn get_transform(&self) -> Option<&Transform2D> {
        self.transform.as_ref()
    }
}

impl HasTransform for TextDrawOptions {
    fn get_transform(&self) -> Option<&Transform2D> {
        self.transform.as_ref()
    }
}
