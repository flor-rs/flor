#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorType {
    Arrow,
    Hand,
    Crosshair,
    Wait,
    Text,
    VerticalResize,
    HorizontalResize,
    Custom(u32), // 自定义指针类型，接受一个整数作为标识
}
