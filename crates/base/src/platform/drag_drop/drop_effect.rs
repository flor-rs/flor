#[derive(Debug, Copy, Clone)]
pub enum DropEffect {
    None,
    Copy,
    Move,
    Link,
    Scroll,
}
