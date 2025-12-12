#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub enum ControlState {
    #[default]
    Normal,
    Focus,
    Hover,
    Active,
    Disable,
}