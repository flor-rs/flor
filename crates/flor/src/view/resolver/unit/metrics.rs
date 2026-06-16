use atomic_float::AtomicF32;

#[derive(Debug)]
pub struct UnitMetrics {
    /// Pixel size of 1rem for this window.
    ///
    /// Initialized from `WindowOption::rem_px`; the default window option is
    /// 16.0, so 1rem resolves to 16px unless the window overrides it.
    pub rem_px: AtomicF32,
    /// Horizontal DPI for this window. The default fallback is 96.0.
    pub dpi_x: AtomicF32,
    /// Vertical DPI for this window. The default fallback is 96.0.
    pub dpi_y: AtomicF32,
    /// Viewport width, taken from the window client area, in pixels.
    pub viewport_width: AtomicF32,
    /// Viewport height, taken from the window client area, in pixels.
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
