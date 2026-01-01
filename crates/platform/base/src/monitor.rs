use std::fmt::Debug;

pub trait MonitorApi: Debug + Clone + Send + Sync + 'static {
    type Monitor;
    type Error;
    type WindowId;
    fn enumerate_monitors() -> Result<Vec<Self::Monitor>, Self::Error>
    where
        Self: Sized;
    fn monitor_from_point(x: i32, y: i32) -> Result<Self::Monitor, Self::Error>;
    fn monitor_from_window_id(window_id: Self::WindowId) -> Result<Self::Monitor, Self::Error>;
    fn name(&self) -> &str;
    fn is_primary(&self) -> bool;
    fn scale_factor(&self) -> f32;
    fn rect(&self) -> (f32, f32, u32, u32);
    fn work_area(&self) -> (f32, f32, u32, u32);
    fn dpi_x(&self) -> f64;
    fn dpi_y(&self) -> f64;
    fn inner(self) -> Self::Monitor;
}
