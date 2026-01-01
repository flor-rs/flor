这是定义

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AlignItems {
/// Items are packed toward the start of the axis
Start,
/// Items are packed toward the end of the axis
End,
/// Items are packed towards the flex-relative start of the axis.
///
/// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
/// to End. In all other cases it is equivalent to Start.
FlexStart,
/// Items are packed towards the flex-relative end of the axis.
///
/// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
/// to Start. In all other cases it is equivalent to End.
FlexEnd,
/// Items are packed along the center of the cross axis
Center,
/// Items are aligned such as their baselines align
Baseline,
/// Stretch to fill the container
Stretch,
}
/// Used to control how child nodes are aligned.
/// Does not apply to Flexbox, and will be ignored if specified on a flex container
/// For Grid it controls alignment in the inline axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/justify-items)
pub type JustifyItems = AlignItems;
/// Controls alignment of an individual node
///
/// Overrides the parent Node's `AlignItems` property.
/// For Flexbox it controls alignment in the cross axis
/// For Grid it controls alignment in the block axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/align-self)
pub type AlignSelf = AlignItems;
/// Controls alignment of an individual node
///
/// Overrides the parent Node's `JustifyItems` property.
/// Does not apply to Flexbox, and will be ignored if specified on a flex child
/// For Grid it controls alignment in the inline axis
///
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/justify-self)
pub type JustifySelf = AlignItems;

pub enum GenericGridPlacement<LineType: GridCoordinate> {
/// Place item according to the auto-placement algorithm, and the parent's grid_auto_flow property
Auto,
/// Place item at specified line (column or row) index
Line(LineType),
/// Item should span specified number of tracks (columns or rows)
Span(u16),
}

GridLine是crate私有的。没有公开，。所以应该参考这个：
pub type GridPlacement = GenericGridPlacement<GridLine>;
impl TaffyAuto for GridPlacement {
const AUTO: Self = Self::Auto;
}
impl TaffyGridLine for GridPlacement {
fn from_line_index(index: i16) -> Self {
GridPlacement::Line(GridLine::from(index))
}
}
impl TaffyGridLine for Line<GridPlacement> {
fn from_line_index(index: i16) -> Self {
Line { start: GridPlacement::from_line_index(index), end: GridPlacement::Auto }
}
}
impl TaffyGridSpan for GridPlacement {
fn from_span(span: u16) -> Self {
GridPlacement::Span(span)
}
}
impl TaffyGridSpan for Line<GridPlacement> {
fn from_span(span: u16) -> Self {
Line { start: GridPlacement::from_span(span), end: GridPlacement::Auto }
}
}

impl Default for GridPlacement {
fn default() -> Self {
Self::Auto
}
}




