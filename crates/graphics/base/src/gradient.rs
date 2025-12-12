use crate::color::Color;

/// 渐变类型，可用于 Gradient Brush
pub enum Gradient {
    /// 线性渐变
    Linear {
        /// 起点坐标
        start: (f32, f32),
        /// 终点坐标
        end: (f32, f32),
        /// 位置/颜色对，位置 0.0~1.0
        colors: Vec<(f32, Color)>,
    },
    /// 径向渐变
    Radial {
        /// 圆心
        center: (f32, f32),
        /// 半径
        radius: f32,
        /// 位置/颜色对
        colors: Vec<(f32, Color)>,
    },
}