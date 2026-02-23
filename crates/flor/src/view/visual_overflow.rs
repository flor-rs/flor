use flor_base::graphics::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum VisualOverflow {
    /// 无溢出（默认值）。
    /// 性能最优，直接使用布局边界进行剔除。
    None,

    /// 统一扩散。
    /// 适用于：居中的阴影、外发光 (Glow)、Focus Ring (焦点框)。
    /// 含义：上下左右都向外扩展 x 像素。
    Uniform(f32),

    /// 自定义四向扩散。
    /// 适用于：带偏移的投影 (Drop Shadow)、不规则装饰、Tooltip 箭头。
    /// 含义：分别指定左、上、右、下的溢出量。
    Custom {
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    },
    Path(Path),
}

// 提供默认值，方便 View trait 使用
impl Default for VisualOverflow {
    fn default() -> Self {
        Self::None
    }
}
