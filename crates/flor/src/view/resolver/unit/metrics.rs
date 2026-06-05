use atomic_float::AtomicF32;

#[derive(Debug)]
pub struct UnitMetrics {
    /// 1rem 对应的像素值 (默认 16.0)
    pub rem_px: AtomicF32,
    /// 水平方向 DPI (默认 96.0)
    pub dpi_x: AtomicF32,
    /// 垂直方向 DPI (默认 96.0)
    pub dpi_y: AtomicF32,
    /// 视口宽度 (窗口客户区宽度，单位: px)
    pub viewport_width: AtomicF32,
    /// 视口高度 (窗口客户区高度，单位: px)
    pub viewport_height: AtomicF32,
}

impl Default for UnitMetrics {
    fn default() -> Self {
        Self {
            rem_px: AtomicF32::new(16.),
            dpi_x: AtomicF32::new(96.),
            dpi_y: AtomicF32::new(96.),
            viewport_width: AtomicF32::new(1024.),
            viewport_height: AtomicF32::new(768.),
        }
    }
}

impl UnitMetrics {
    pub fn new(
        dpi_x: f32,
        dpi_y: f32,
        rem_px: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> UnitMetrics {
        Self {
            rem_px: AtomicF32::new(rem_px),
            dpi_x: AtomicF32::new(dpi_x),
            dpi_y: AtomicF32::new(dpi_y),
            viewport_width: AtomicF32::new(viewport_width),
            viewport_height: AtomicF32::new(viewport_height),
        }
    }
}
