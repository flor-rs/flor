use crate::view::control_state::ControlState;
#[cfg(feature = "layout-block")]
use taffy::TextAlign;
#[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
use taffy::{AlignContent, AlignItems, AlignSelf, JustifyContent, JustifyItems};
use taffy::{
    BoxSizing, Dimension, Display, LengthPercentage, LengthPercentageAuto, Line, Overflow, Point,
    Position, Rect, Size,
};
#[cfg(feature = "layout-flex")]
use taffy::{FlexDirection, FlexWrap};
#[cfg(feature = "layout-grid")]
use taffy::{
    GridAutoFlow, GridPlacement, Line, NonRepeatedTrackSizingFunction, TrackSizingFunction,
};

#[derive(Clone, Debug)]
pub enum Layout {
    /// What layout strategy should be used?
    Display(Display),
    /// Whether a child is display:table or not. This affects children of block layouts.
    /// This should really be part of `Display`, but it is currently seperate because table layout isn't implemented
    ItemIsTable(bool),
    /// Should size styles apply to the content box or the border box of the node
    BoxSizing(BoxSizing),

    // Overflow properties
    /// How children overflowing their container should affect layout
    Overflow(Point<Overflow>),
    /// How much space (in points) should be reserved for the scrollbars of `Overflow::Scroll` and `Overflow::Auto` nodes.
    ScrollbarWidth(f32),

    // Position properties
    /// What should the `position` value of this struct use as a base offset?
    Position(Position),
    /// How should the position of this element be tweaked relative to the layout defined?
    Inset(Rect<LengthPercentageAuto>),

    // Size properties
    /// Sets the initial size of the item
    Size(Size<Dimension>),
    /// Controls the minimum size of the item
    MinSize(Size<Dimension>),
    /// Controls the maximum size of the item
    MaxSize(Size<Dimension>),
    /// Sets the preferred aspect ratio for the item
    ///
    /// The ratio is calculated as width divided by height.
    AspectRatio(f32),

    // Spacing Properties
    /// How large should the margin be on each side?
    Margin(Rect<LengthPercentageAuto>),
    /// How large should the padding be on each side?
    Padding(Rect<LengthPercentage>),

    /// How large should the border be on each side?
    Border(Rect<LengthPercentage>),

    // Alignment properties
    /// How this node's children aligned in the cross/block axis?
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    AlignItems(AlignItems),
    /// How this node should be aligned in the cross/block axis
    /// Falls back to the parents [`AlignItems`] if not set
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    AlignSelf(AlignSelf),
    /// How this node's children should be aligned in the inline axis
    #[cfg(feature = "layout-grid")]
    JustifyItems(AlignItems),
    /// How this node should be aligned in the inline axis
    /// Falls back to the parents [`JustifyItems`] if not set
    #[cfg(feature = "layout-grid")]
    JustifySelf(AlignSelf),
    /// How should content contained within this item be aligned in the cross/block axis
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    AlignContent(AlignContent),
    /// How should content contained within this item be aligned in the main/inline axis
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    JustifyContent(JustifyContent),
    /// How large should the gaps between items in a grid or flex container be?
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    Gap(Size<LengthPercentage>),

    // Block container properties
    /// How items elements should aligned in the inline axis
    #[cfg(feature = "layout-block")]
    TextAlign(TextAlign),

    // Flexbox container properties
    /// Which direction does the main axis flow in?
    #[cfg(feature = "layout-flex")]
    FlexDirection(FlexDirection),
    /// Should elements wrap, or stay in a single line?
    #[cfg(feature = "layout-flex")]
    FlexWrap(FlexWrap),

    // Flexbox item properties
    /// Sets the initial main axis size of the item
    #[cfg(feature = "layout-flex")]
    FlexBasis(Dimension),
    /// The relative rate at which this item grows when it is expanding to fill space
    ///
    /// 0.0 is the default value, and this value must be positive.
    #[cfg(feature = "layout-flex")]
    FlexGrow(f32),
    /// The relative rate at which this item shrinks when it is contracting to fit into space
    ///
    /// 1.0 is the default value, and this value must be positive.
    #[cfg(feature = "layout-flex")]
    FlexShrink(f32),

    // Grid container properies
    /// Defines the track sizing functions (heights) of the grid rows
    #[cfg(feature = "layout-grid")]
    GridTemplateRows(Vec<TrackSizingFunction>),
    /// Defines the track sizing functions (widths) of the grid columns
    #[cfg(feature = "layout-grid")]
    GridTemplateColumns(Vec<TrackSizingFunction>),
    /// Defines the size of implicitly created rows
    #[cfg(feature = "layout-grid")]
    GridAutoRows(Vec<NonRepeatedTrackSizingFunction>),
    /// Defined the size of implicitly created columns
    #[cfg(feature = "layout-grid")]
    GridAutoColumns(Vec<NonRepeatedTrackSizingFunction>),
    /// Controls how items get placed into the grid for auto-placed items
    #[cfg(feature = "layout-grid")]
    GridAutoFlow(GridAutoFlow),

    // Grid child properties
    /// Defines which row in the grid the item should start and end at
    #[cfg(feature = "layout-grid")]
    GridRow(Line<GridPlacement>),
    /// Defines which column in the grid the item should start and end at
    #[cfg(feature = "layout-grid")]
    GridColumn(Line<GridPlacement>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayoutKey {
    Display,
    ItemIsTable,
    BoxSizing,
    Overflow,
    ScrollbarWidth,
    Position,
    Inset,
    Size,
    MinSize,
    MaxSize,
    AspectRatio,
    Margin,
    Padding,
    Border,
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    AlignItems,
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    AlignSelf,
    #[cfg(feature = "layout-grid")]
    JustifyItems,
    #[cfg(feature = "layout-grid")]
    JustifySelf,
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    AlignContent,
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    JustifyContent,
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    Gap,
    #[cfg(feature = "layout-block")]
    TextAlign,
    #[cfg(feature = "layout-flex")]
    FlexDirection,
    #[cfg(feature = "layout-flex")]
    FlexWrap,
    #[cfg(feature = "layout-flex")]
    FlexBasis,
    #[cfg(feature = "layout-flex")]
    FlexGrow,
    #[cfg(feature = "layout-flex")]
    FlexShrink,
    #[cfg(feature = "layout-grid")]
    GridTemplateRows,
    #[cfg(feature = "layout-grid")]
    GridTemplateColumns,
    #[cfg(feature = "layout-grid")]
    GridAutoRows,
    #[cfg(feature = "layout-grid")]
    GridAutoColumns,
    #[cfg(feature = "layout-grid")]
    GridAutoFlow,
    #[cfg(feature = "layout-grid")]
    GridRow,
    #[cfg(feature = "layout-grid")]
    GridColumn,
}

pub trait LayoutStateSelectorExt: Sized {
    fn display(self, value: Display) -> Self;
    fn item_is_table(self, value: bool) -> Self;
    fn box_sizing(self, value: BoxSizing) -> Self;
    fn overflow(self, value: Point<Overflow>) -> Self;
    fn scrollbar_width(self, value: f32) -> Self;
    fn position(self, value: Position) -> Self;
    fn inset(self, value: Rect<LengthPercentageAuto>) -> Self;
    fn size(self, value: Size<Dimension>) -> Self;
    fn min_size(self, value: Size<Dimension>) -> Self;
    fn max_size(self, value: Size<Dimension>) -> Self;
    fn aspect_ratio(self, value: f32) -> Self;
    fn margin(self, value: Rect<LengthPercentageAuto>) -> Self;
    fn padding(self, value: Rect<LengthPercentage>) -> Self;
    fn border(self, value: Rect<LengthPercentage>) -> Self;
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn align_items(self, value: AlignItems) -> Self;
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn align_self(self, value: AlignSelf) -> Self;
    #[cfg(feature = "layout-grid")]
    fn justify_items(self, value: AlignItems) -> Self;
    #[cfg(feature = "layout-grid")]
    fn justify_self(self, value: AlignSelf) -> Self;
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn align_content(self, value: AlignContent) -> Self;
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn justify_content(self, value: JustifyContent) -> Self;
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn gap(self, value: Size<LengthPercentage>) -> Self;
    #[cfg(feature = "layout-block")]
    fn text_align(self, value: TextAlign) -> Self;
    #[cfg(feature = "layout-flex")]
    fn flex_direction(self, value: FlexDirection) -> Self;
    #[cfg(feature = "layout-flex")]
    fn flex_wrap(self, value: FlexWrap) -> Self;
    #[cfg(feature = "layout-flex")]
    fn flex_basis(self, value: Dimension) -> Self;
    #[cfg(feature = "layout-flex")]
    fn flex_grow(self, value: f32) -> Self;
    #[cfg(feature = "layout-flex")]
    fn flex_shrink(self, value: f32) -> Self;
    #[cfg(feature = "layout-grid")]
    fn grid_template_rows(self, value: Vec<TrackSizingFunction>) -> Self;
    #[cfg(feature = "layout-grid")]
    fn grid_template_columns(self, value: Vec<TrackSizingFunction>) -> Self;
    #[cfg(feature = "layout-grid")]
    fn grid_auto_rows(self, value: Vec<NonRepeatedTrackSizingFunction>) -> Self;
    #[cfg(feature = "layout-grid")]
    fn grid_auto_columns(self, value: Vec<NonRepeatedTrackSizingFunction>) -> Self;
    #[cfg(feature = "layout-grid")]
    fn grid_auto_flow(self, value: GridAutoFlow) -> Self;
    #[cfg(feature = "layout-grid")]
    fn grid_row(self, value: Line<GridPlacement>) -> Self;
    #[cfg(feature = "layout-grid")]
    fn grid_column(self, value: Line<GridPlacement>) -> Self;
}
impl LayoutStateSelectorExt for StateSelector<LayoutKey, Layout> {
    fn display(mut self, value: Display) -> Self {
        self.push(LayoutKey::Display, Layout::Display(value));
        self
    }
    fn item_is_table(mut self, value: bool) -> Self {
        self.push(LayoutKey::ItemIsTable, Layout::ItemIsTable(value));
        self
    }
    fn box_sizing(mut self, value: BoxSizing) -> Self {
        self.push(LayoutKey::BoxSizing, Layout::BoxSizing(value));
        self
    }
    fn overflow(mut self, value: Point<Overflow>) -> Self {
        self.push(LayoutKey::Overflow, Layout::Overflow(value));
        self
    }
    fn scrollbar_width(mut self, value: f32) -> Self {
        self.push(LayoutKey::ScrollbarWidth, Layout::ScrollbarWidth(value));
        self
    }
    fn position(mut self, value: Position) -> Self {
        self.push(LayoutKey::Position, Layout::Position(value));
        self
    }
    fn inset(mut self, value: Rect<LengthPercentageAuto>) -> Self {
        self.push(LayoutKey::Inset, Layout::Inset(value));
        self
    }
    fn size(mut self, value: Size<Dimension>) -> Self {
        self.push(LayoutKey::Size, Layout::Size(value));
        self
    }
    fn min_size(mut self, value: Size<Dimension>) -> Self {
        self.push(LayoutKey::MinSize, Layout::MinSize(value));
        self
    }
    fn max_size(mut self, value: Size<Dimension>) -> Self {
        self.push(LayoutKey::MaxSize, Layout::MaxSize(value));
        self
    }
    fn aspect_ratio(mut self, value: f32) -> Self {
        self.push(LayoutKey::AspectRatio, Layout::AspectRatio(value));
        self
    }
    fn margin(mut self, value: Rect<LengthPercentageAuto>) -> Self {
        self.push(LayoutKey::Margin, Layout::Margin(value));
        self
    }
    fn padding(mut self, value: Rect<LengthPercentage>) -> Self {
        self.push(LayoutKey::Padding, Layout::Padding(value));
        self
    }

    fn border(mut self, value: Rect<LengthPercentage>) -> Self {
        self.push(LayoutKey::Border, Layout::Border(value));
        self
    }
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn align_items(mut self, value: AlignItems) -> Self {
        self.push(LayoutKey::AlignItems, Layout::AlignItems(value));
        self
    }
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn align_self(mut self, value: AlignSelf) -> Self {
        self.push(LayoutKey::AlignSelf, Layout::AlignSelf(value));
        self
    }
    #[cfg(feature = "layout-grid")]
    fn justify_items(mut self, value: AlignItems) -> Self {
        self.push(LayoutKey::JustifyItems, Layout::JustifyItems(value));
        self
    }
    #[cfg(feature = "layout-grid")]
    fn justify_self(mut self, value: AlignSelf) -> Self {
        self.push(LayoutKey::JustifySelf, Layout::JustifySelf(value));
        self
    }
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn align_content(mut self, value: AlignContent) -> Self {
        self.push(LayoutKey::AlignContent, Layout::AlignContent(value));
        self
    }
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn justify_content(mut self, value: JustifyContent) -> Self {
        self.push(LayoutKey::JustifyContent, Layout::JustifyContent(value));
        self
    }
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    fn gap(mut self, value: Size<LengthPercentage>) -> Self {
        self.push(LayoutKey::Gap, Layout::Gap(value));
        self
    }
    #[cfg(feature = "layout-block")]
    fn text_align(mut self, value: TextAlign) -> Self {
        self.push(LayoutKey::TextAlign, Layout::TextAlign(value));
        self
    }
    #[cfg(feature = "layout-flex")]
    fn flex_direction(mut self, value: FlexDirection) -> Self {
        self.push(LayoutKey::FlexDirection, Layout::FlexDirection(value));
        self
    }
    #[cfg(feature = "layout-flex")]
    fn flex_wrap(mut self, value: FlexWrap) -> Self {
        self.push(LayoutKey::FlexWrap, Layout::FlexWrap(value));
        self
    }
    #[cfg(feature = "layout-flex")]
    fn flex_basis(mut self, value: Dimension) -> Self {
        self.push(LayoutKey::FlexBasis, Layout::FlexBasis(value));
        self
    }
    #[cfg(feature = "layout-flex")]
    fn flex_grow(mut self, value: f32) -> Self {
        self.push(LayoutKey::FlexGrow, Layout::FlexGrow(value));
        self
    }
    #[cfg(feature = "layout-flex")]
    fn flex_shrink(mut self, value: f32) -> Self {
        self.push(LayoutKey::FlexShrink, Layout::FlexShrink(value));
        self
    }
    #[cfg(feature = "layout-grid")]
    fn grid_template_rows(mut self, value: Vec<TrackSizingFunction>) -> Self {
        self.push(LayoutKey::GridTemplateRows, Layout::GridTemplateRows(value));
        self
    }
    #[cfg(feature = "layout-grid")]
    fn grid_template_columns(mut self, value: Vec<TrackSizingFunction>) -> Self {
        self.push(
            LayoutKey::GridTemplateColumns,
            Layout::GridTemplateColumns(value),
        );
        self
    }
    #[cfg(feature = "layout-grid")]
    fn grid_auto_rows(mut self, value: Vec<NonRepeatedTrackSizingFunction>) -> Self {
        self.push(LayoutKey::GridAutoRows, Layout::GridAutoRows(value));
        self
    }
    #[cfg(feature = "layout-grid")]
    fn grid_auto_columns(mut self, value: Vec<NonRepeatedTrackSizingFunction>) -> Self {
        self.push(LayoutKey::GridAutoColumns, Layout::GridAutoColumns(value));
        self
    }
    #[cfg(feature = "layout-grid")]
    fn grid_auto_flow(mut self, value: GridAutoFlow) -> Self {
        self.push(LayoutKey::GridAutoFlow, Layout::GridAutoFlow(value));
        self
    }
    #[cfg(feature = "layout-grid")]
    fn grid_row(mut self, value: Line<GridPlacement>) -> Self {
        self.push(LayoutKey::GridRow, Layout::GridRow(value));
        self
    }
    #[cfg(feature = "layout-grid")]
    fn grid_column(mut self, value: Line<GridPlacement>) -> Self {
        self.push(LayoutKey::GridColumn, Layout::GridColumn(value));
        self
    }
}
use crate::view::style::style_selector::StateSelector;

pub type LayoutStateSelector = StateSelector<LayoutKey, Layout>;

pub trait CalcTaffyStyle {
    fn calc_taffy_style(&self, control_state: ControlState) -> Option<taffy::Style>;
}

impl CalcTaffyStyle for LayoutStateSelector {
    fn calc_taffy_style(&self, control_state: ControlState) -> Option<taffy::Style> {
        if !self.is_dirty(control_state) {
            return None;
        }
        let mut layout_style = taffy::Style::default();

        // #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
        // pub enum ControlState {
        //     #[default]
        //     Normal,
        //     Focus,
        //     Hover,
        //     Active,
        //     Disable,
        // }

        // 这里少逻辑了。 如果访问的不是normal map，继承成normalmap的数据+新的数据覆盖进来

        if let Some(map) = self.get_style(control_state) {
            for layout in map.values() {
                match layout {
                    Layout::Display(v) => layout_style.display = *v,
                    Layout::ItemIsTable(x) => layout_style.item_is_table = *x,
                    Layout::BoxSizing(v) => layout_style.box_sizing = *v,
                    Layout::Overflow(v) => layout_style.overflow = *v,
                    Layout::ScrollbarWidth(v) => layout_style.scrollbar_width = *v,
                    Layout::Position(v) => layout_style.position = *v,
                    Layout::Inset(v) => layout_style.inset = *v,
                    Layout::Size(v) => layout_style.size = *v,
                    Layout::MinSize(v) => layout_style.min_size = *v,
                    Layout::MaxSize(v) => layout_style.max_size = *v,
                    Layout::AspectRatio(v) => layout_style.aspect_ratio = Some(*v),
                    Layout::Margin(v) => layout_style.margin = *v,
                    Layout::Padding(v) => layout_style.padding = *v,
                    Layout::Border(v) => layout_style.border = *v,
                    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
                    Layout::AlignItems(v) => layout_style.align_items = Some(*v),
                    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
                    Layout::AlignSelf(v) => layout_style.align_self = Some(*v),
                    #[cfg(feature = "layout-grid")]
                    Layout::JustifyItems(v) => layout_style.justify_items = Some(*v),
                    #[cfg(feature = "layout-grid")]
                    Layout::JustifySelf(v) => layout_style.justify_self = Some(*v),
                    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
                    Layout::AlignContent(v) => layout_style.align_content = Some(*v),
                    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
                    Layout::JustifyContent(v) => layout_style.justify_content = Some(*v),
                    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
                    Layout::Gap(v) => layout_style.gap = *v,
                    #[cfg(feature = "layout-block")]
                    Layout::TextAlign(v) => layout_style.text_align = *v,
                    #[cfg(feature = "layout-flex")]
                    Layout::FlexDirection(v) => layout_style.flex_direction = *v,
                    #[cfg(feature = "layout-flex")]
                    Layout::FlexWrap(v) => layout_style.flex_wrap = *v,
                    #[cfg(feature = "layout-flex")]
                    Layout::FlexBasis(v) => layout_style.flex_basis = *v,
                    #[cfg(feature = "layout-flex")]
                    Layout::FlexGrow(v) => layout_style.flex_grow = *v,
                    #[cfg(feature = "layout-flex")]
                    Layout::FlexShrink(v) => layout_style.flex_shrink = *v,
                    #[cfg(feature = "layout-grid")]
                    Layout::GridTemplateRows(v) => layout_style.grid_template_rows = v.clone(),
                    #[cfg(feature = "layout-grid")]
                    Layout::GridTemplateColumns(v) => {
                        layout_style.grid_template_columns = v.clone()
                    }
                    #[cfg(feature = "layout-grid")]
                    Layout::GridAutoRows(v) => layout_style.grid_auto_rows = v.clone(),
                    #[cfg(feature = "layout-grid")]
                    Layout::GridAutoColumns(v) => layout_style.grid_auto_columns = v.clone(),
                    #[cfg(feature = "layout-grid")]
                    Layout::GridAutoFlow(v) => layout_style.grid_auto_flow = *v,
                    #[cfg(feature = "layout-grid")]
                    Layout::GridRow(v) => layout_style.grid_row = *v,
                    #[cfg(feature = "layout-grid")]
                    Layout::GridColumn(v) => layout_style.grid_column = *v,
                }
            }
        }
        Some(layout_style)
    }
}
