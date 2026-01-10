#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FramePolicy {
    Always,
    #[default]
    VisibleOnly,
}
