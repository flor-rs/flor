use std::fmt::Debug;

pub trait MonitorApi: Debug + Clone + Send + Sync + 'static {
    type Monitor;
    type Error;
    fn enumerate_monitors() -> Result<Vec<Self::Monitor>, Self::Error>
    where
        Self: Sized;
    fn monitor_from_point(x: i32, y: i32) -> Result<Self::Monitor, Self::Error>;
    fn name(&self) -> &str;
    fn is_primary(&self) -> bool;
    fn scale_factor(&self) -> f32;
    fn rect(&self) -> (f32, f32, u32, u32);
    fn work_area(&self) -> (f32, f32, u32, u32);
    fn inner(self) -> Self::Monitor;
}
