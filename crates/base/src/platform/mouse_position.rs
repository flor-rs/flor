#[derive(Copy, Clone, Debug)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

impl Default for MousePosition {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}
