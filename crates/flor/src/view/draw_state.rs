#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawState {
    Disabled,
    Pressed,
    Hover,
    Normal,
}