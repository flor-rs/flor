use crate::WindowMode;

pub trait WindowApi {
    type Error;
    type Cursor;

    /// 创建窗口
    /// title: 窗口标题
    /// width, height: 初始客户区大小 (逻辑像素或物理像素，取决于你的框架约定，通常建议物理像素)
    fn create_window(title: &str, width: u32, height: u32) -> Result<Self, Self::Error>
    where
        Self: Sized;

    // --- 核心绘制 ---

    /// 同步强制更新 (Windows 特供)
    /// Windows: 立即调用 WndProc 绘制 (用于初始化防白屏)
    /// macOS/Linux: 空实现或只更新标记
    fn update_window(&self) -> Result<(), Self::Error>;

    // --- 可见性 (Visibility) ---

    /// 显示窗口
    fn show(&self) -> Result<(), Self::Error>;

    /// 隐藏窗口
    fn hide(&self) -> Result<(), Self::Error>;

    // --- 窗口状态 (State) ---

    /// 设置窗口模式 (普通、最大化、最小化、全屏)
    fn set_window_mode(&self, mode: WindowMode) -> Result<(), Self::Error>;

    /// 获取当前窗口模式
    fn get_window_mode(&self) -> Result<WindowMode, Self::Error>;

    // --- 屏幕与 DPI ---

    /// 获取 DPI 缩放因子
    /// 返回 1.0 (标准), 1.25 (125%), 2.0 (Retina) 等
    fn get_scale_factor(&self) -> Result<f32, Self::Error>;
    fn get_dpi(&self) -> Result<(f64, f64), Self::Error>;

    // --- 位置 (Position) - i32 ---
    // 允许负数，支持多显示器负坐标

    fn get_left(&self) -> Result<i32, Self::Error>;
    fn get_top(&self) -> Result<i32, Self::Error>;
    fn set_left(&self, left: i32) -> Result<(), Self::Error>;
    fn set_top(&self, top: i32) -> Result<(), Self::Error>;

    /// 同时设置位置 (x, y)
    fn set_position(&self, pos: (i32, i32)) -> Result<(), Self::Error>;

    // --- 尺寸 (Size) - u32 ---
    // 物理尺寸不可能为负

    fn get_width(&self) -> Result<u32, Self::Error>;
    fn get_height(&self) -> Result<u32, Self::Error>;
    fn set_width(&self, width: u32) -> Result<(), Self::Error>;
    fn set_height(&self, height: u32) -> Result<(), Self::Error>;

    /// 同时设置大小 (width, height)
    fn set_size(&self, size: (u32, u32)) -> Result<(), Self::Error>;

    // --- 区域查询 (Rects) ---

    /// 获取客户区大小 (Client Size)
    /// 用途：重置渲染后端 (D2D/WGPU) 的缓冲区大小
    /// 返回：(width, height)
    fn get_client_size(&self) -> Result<(u32, u32), Self::Error>;

    /// 获取客户区在屏幕上的绝对矩形 (Client Rect in Screen Coords)
    /// 用途：计算弹出菜单(Popup)、输入法(IME)候选框的位置
    /// 返回：(screen_x, screen_y, width, height)
    /// 注意：x, y 是 i32 (可能为负)，w, h 是 u32
    fn get_client_rect(&self) -> Result<(i32, i32, u32, u32), Self::Error>;

    /// 获取整个窗口(含标题栏边框)的矩形
    /// 用途：保存窗口位置配置、对齐其他窗口
    /// 返回：(screen_x, screen_y, width, height)
    fn get_window_rect(&self) -> Result<(i32, i32, u32, u32), Self::Error>;

    fn drag_window(&self) -> Result<(), Self::Error>;

    // ime
    fn set_ime_window_location(&self, rect: (i32, i32, u32, u32)) -> Result<(), Self::Error>;
    fn set_ime_open_state(&self, is_open: bool) -> Result<(), Self::Error>;
    fn set_ime_allowed(&self, allow: bool) -> Result<(), Self::Error>;

    // cursor

    fn set_cursor(cursor: Option<Self::Cursor>) -> Result<(), Self::Error>;

    // --- 生命周期 ---

    fn destroy(&self) -> Result<(), Self::Error>;
}

pub trait WindowOperations {
    type Error;
    /// 异步请求重绘 (推荐)
    /// 所有平台通用：标记窗口为脏，等待下一次事件循环绘制
    fn request_redraw(&self) -> Result<(), Self::Error>;
    // cursor
    fn capture_mouse(&self) -> Result<(), Self::Error>;
    fn release_mouse(&self) -> Result<(), Self::Error>;
}
