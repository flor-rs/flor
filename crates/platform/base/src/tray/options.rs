use crate::IconSource;

/// 托盘图标定义
pub struct TrayOptions {
    pub icon_path: Option<IconSource>, // 或直接传 Icon 句柄/二进制
    pub tooltip: String,
    // 菜单通常单独处理，因为菜单是复杂的层级结构
}
