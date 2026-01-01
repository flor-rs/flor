use crate::view::class::ClassLoader;
use crate::view::control_state::ControlState;
use crate::view::state_selector::StateSelector;
#[cfg(feature = "layout-grid")]
use taffy::style_helpers::TaffyGridLine;
#[cfg(feature = "layout-block")]
use taffy::TextAlign;
#[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
use taffy::{AlignContent, AlignItems, AlignSelf, JustifyContent};
use taffy::{
    BoxSizing, Dimension, Display, LengthPercentage, LengthPercentageAuto, Overflow, Point,
    Position, Rect, Size, Style,
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

pub type LayoutStateSelector = StateSelector<LayoutKey, Layout>;

pub trait CalcTaffyStyle {
    fn calc_update_taffy_style(&self, control_state: ControlState) -> Option<Style>;
    fn calc_taffy_style(&self, control_state: ControlState) -> Style;
}

impl CalcTaffyStyle for LayoutStateSelector {
    fn calc_update_taffy_style(&self, control_state: ControlState) -> Option<Style> {
        if !self.is_dirty(control_state) {
            return None;
        }
        Some(self.calc_taffy_style(control_state))
    }

    fn calc_taffy_style(&self, control_state: ControlState) -> Style {
        let mut layout_style = Style::default();

        // #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
        // pub enum ControlState {
        //     #[default]
        //     Normal,
        //     Focus,
        //     Hover,
        //     Active,
        //     Disable,
        // }

        // 这里少逻辑了。 如果访问的不是normal map，继承成normal map的数据+新的数据覆盖进来

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
        layout_style
    }
}

// Helper functions for parsing
fn extract_bracket_value(s: &str) -> Option<&str> {
    if s.starts_with('[') && s.ends_with(']') {
        Some(&s[1..s.len() - 1])
    } else {
        None
    }
}

/// Unit conversion configuration
#[derive(Clone, Copy)]
struct UnitConfig {
    rem_px: f32,   // 1rem = rem_px pixels
    pt_px: f32,    // 1pt = pt_px pixels (calculated from dpi)
}

impl Default for UnitConfig {
    fn default() -> Self {
        Self {
            rem_px: 16.0,
            pt_px: 1.333333, // 96dpi default: 96/72
        }
    }
}

impl UnitConfig {
    fn from_selector(selector: &LayoutStateSelector) -> Self {
        use std::sync::atomic::Ordering;
        let rem_px = selector.rem_px.load(Ordering::Acquire);
        let dpi = selector.dpi_y.load(Ordering::Acquire) as f32;
        let pt_px = dpi / 72.0; // 1pt = dpi/72 pixels
        Self { rem_px, pt_px }
    }
}

fn parse_length_percentage_auto(value: &str, cfg: &UnitConfig) -> Option<LengthPercentageAuto> {
    if value == "auto" {
        return Some(LengthPercentageAuto::Auto);
    }
    if value == "full" || value == "screen" {
        return Some(LengthPercentageAuto::Percent(100.0));
    }
    if let Some(v) = value.strip_suffix('%') {
        if let Ok(n) = v.parse::<f32>() {
            return Some(LengthPercentageAuto::Percent(n));
        }
    }
    if let Some(v) = value.strip_suffix("px") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(LengthPercentageAuto::Length(n));
        }
    }
    if let Some(v) = value.strip_suffix("rem") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(LengthPercentageAuto::Length(n * cfg.rem_px));
        }
    }
    if let Some(v) = value.strip_suffix("pt") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(LengthPercentageAuto::Length(n * cfg.pt_px));
        }
    }
    if let Ok(n) = value.parse::<f32>() {
        return Some(LengthPercentageAuto::Length(n * 4.0));
    }
    if value.contains('/') {
        let parts: Vec<&str> = value.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(num), Ok(den)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                if den != 0.0 {
                    return Some(LengthPercentageAuto::Percent(num / den * 100.0));
                }
            }
        }
    }
    None
}

fn parse_length_percentage(value: &str, cfg: &UnitConfig) -> Option<LengthPercentage> {
    if value == "full" {
        return Some(LengthPercentage::Percent(100.0));
    }
    if let Some(v) = value.strip_suffix('%') {
        if let Ok(n) = v.parse::<f32>() {
            return Some(LengthPercentage::Percent(n));
        }
    }
    if let Some(v) = value.strip_suffix("px") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(LengthPercentage::Length(n));
        }
    }
    if let Some(v) = value.strip_suffix("rem") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(LengthPercentage::Length(n * cfg.rem_px));
        }
    }
    if let Some(v) = value.strip_suffix("pt") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(LengthPercentage::Length(n * cfg.pt_px));
        }
    }
    if let Ok(n) = value.parse::<f32>() {
        return Some(LengthPercentage::Length(n * 4.0));
    }
    None
}

fn parse_dimension(value: &str, cfg: &UnitConfig) -> Option<Dimension> {
    if value == "auto" || value == "fit" || value == "min" || value == "max" {
        return Some(Dimension::Auto);
    }
    if value == "full" || value == "screen" {
        return Some(Dimension::Percent(100.0));
    }
    if let Some(v) = value.strip_suffix('%') {
        if let Ok(n) = v.parse::<f32>() {
            return Some(Dimension::Percent(n));
        }
    }
    if let Some(v) = value.strip_suffix("px") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(Dimension::Length(n));
        }
    }
    if let Some(v) = value.strip_suffix("rem") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(Dimension::Length(n * cfg.rem_px));
        }
    }
    if let Some(v) = value.strip_suffix("pt") {
        if let Ok(n) = v.parse::<f32>() {
            return Some(Dimension::Length(n * cfg.pt_px));
        }
    }
    if let Ok(n) = value.parse::<f32>() {
        return Some(Dimension::Length(n * 4.0));
    }
    if value.contains('/') {
        let parts: Vec<&str> = value.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(num), Ok(den)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                if den != 0.0 {
                    return Some(Dimension::Percent(num / den * 100.0));
                }
            }
        }
    }
    None
}

#[derive(Default)]
#[allow(dead_code)] // Fields may be unused depending on feature flags
struct LayoutAccumulator {
    updates: Vec<(LayoutKey, Layout)>,

    // Padding
    pl: Option<LengthPercentage>,
    pr: Option<LengthPercentage>,
    pt: Option<LengthPercentage>,
    pb: Option<LengthPercentage>,

    // Margin
    ml: Option<LengthPercentageAuto>,
    mr: Option<LengthPercentageAuto>,
    mt: Option<LengthPercentageAuto>,
    mb: Option<LengthPercentageAuto>,

    // Inset
    il: Option<LengthPercentageAuto>,
    ir: Option<LengthPercentageAuto>,
    it: Option<LengthPercentageAuto>,
    ib: Option<LengthPercentageAuto>,

    // Size
    w: Option<Dimension>,
    h: Option<Dimension>,
    min_w: Option<Dimension>,
    min_h: Option<Dimension>,
    max_w: Option<Dimension>,
    max_h: Option<Dimension>,

    // Gap
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    gap_w: Option<LengthPercentage>,
    #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
    gap_h: Option<LengthPercentage>,

    // Overflow
    of_x: Option<Overflow>,
    of_y: Option<Overflow>,

    // Grid Lines
    #[cfg(feature = "layout-grid")]
    row_start: Option<GridPlacement>,
    #[cfg(feature = "layout-grid")]
    row_end: Option<GridPlacement>,
    #[cfg(feature = "layout-grid")]
    col_start: Option<GridPlacement>,
    #[cfg(feature = "layout-grid")]
    col_end: Option<GridPlacement>,
}

impl LayoutAccumulator {
    fn parse(&mut self, class: &str, cfg: &UnitConfig) {
        let class = class.trim();
        if class.is_empty() {
            return;
        }

        // Specific prefixes first

        // === Padding (Specific) ===
        if let Some(suffix) = class.strip_prefix("pl-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage(v, cfg))
                .or_else(|| parse_length_percentage(suffix, cfg))
            {
                self.pl = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("pr-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage(v, cfg))
                .or_else(|| parse_length_percentage(suffix, cfg))
            {
                self.pr = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("pt-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage(v, cfg))
                .or_else(|| parse_length_percentage(suffix, cfg))
            {
                self.pt = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("pb-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage(v, cfg))
                .or_else(|| parse_length_percentage(suffix, cfg))
            {
                self.pb = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("px-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage(v, cfg))
                .or_else(|| parse_length_percentage(suffix, cfg))
            {
                self.pl = Some(v);
                self.pr = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("py-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage(v, cfg))
                .or_else(|| parse_length_percentage(suffix, cfg))
            {
                self.pt = Some(v);
                self.pb = Some(v);
                return;
            }
        }
        // === Padding (Generic) ===
        if let Some(suffix) = class.strip_prefix("p-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage(v, cfg))
                .or_else(|| parse_length_percentage(suffix, cfg))
            {
                self.pl = Some(v);
                self.pr = Some(v);
                self.pt = Some(v);
                self.pb = Some(v);
                return;
            }
        }

        // === Margin (Specific) ===
        if let Some(suffix) = class.strip_prefix("ml-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.ml = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("mr-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.mr = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("mt-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.mt = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("mb-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.mb = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("mx-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.ml = Some(v);
                self.mr = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("my-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.mt = Some(v);
                self.mb = Some(v);
                return;
            }
        }
        // === Margin (Generic) ===
        if let Some(suffix) = class.strip_prefix("m-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.ml = Some(v);
                self.mr = Some(v);
                self.mt = Some(v);
                self.mb = Some(v);
                return;
            }
        }

        // === Inset (Specific) ===
        if let Some(suffix) = class.strip_prefix("left-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.il = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("right-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.ir = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("top-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.it = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("bottom-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.ib = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("inset-x-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.il = Some(v);
                self.ir = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("inset-y-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.it = Some(v);
                self.ib = Some(v);
                return;
            }
        }
        // === Inset (Generic) ===
        if let Some(suffix) = class.strip_prefix("inset-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_length_percentage_auto(v, cfg))
                .or_else(|| parse_length_percentage_auto(suffix, cfg))
            {
                self.il = Some(v);
                self.ir = Some(v);
                self.it = Some(v);
                self.ib = Some(v);
                return;
            }
        }

        // === Sizing ===
        if let Some(suffix) = class.strip_prefix("w-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_dimension(v, cfg))
                .or_else(|| parse_dimension(suffix, cfg))
            {
                self.w = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("h-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_dimension(v, cfg))
                .or_else(|| parse_dimension(suffix, cfg))
            {
                self.h = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("min-w-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_dimension(v, cfg))
                .or_else(|| parse_dimension(suffix, cfg))
            {
                self.min_w = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("min-h-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_dimension(v, cfg))
                .or_else(|| parse_dimension(suffix, cfg))
            {
                self.min_h = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("max-w-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_dimension(v, cfg))
                .or_else(|| parse_dimension(suffix, cfg))
            {
                self.max_w = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("max-h-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_dimension(v, cfg))
                .or_else(|| parse_dimension(suffix, cfg))
            {
                self.max_h = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("size-") {
            if let Some(v) = extract_bracket_value(suffix)
                .and_then(|v| parse_dimension(v, cfg))
                .or_else(|| parse_dimension(suffix, cfg))
            {
                self.w = Some(v);
                self.h = Some(v);
                return;
            }
        }

        // === Gap ===
        #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
        {
            if let Some(suffix) = class.strip_prefix("gap-x-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|v| parse_length_percentage(v, cfg))
                    .or_else(|| parse_length_percentage(suffix, cfg))
                {
                    self.gap_w = Some(v);
                    return;
                }
            }
            if let Some(suffix) = class.strip_prefix("gap-y-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|v| parse_length_percentage(v, cfg))
                    .or_else(|| parse_length_percentage(suffix, cfg))
                {
                    self.gap_h = Some(v);
                    return;
                }
            }
            if let Some(suffix) = class.strip_prefix("gap-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|v| parse_length_percentage(v, cfg))
                    .or_else(|| parse_length_percentage(suffix, cfg))
                {
                    self.gap_w = Some(v);
                    self.gap_h = Some(v);
                    return;
                }
            }
        }

        // === Overflow ===
        if let Some(suffix) = class.strip_prefix("overflow-x-") {
            match suffix {
                "visible" => self.of_x = Some(Overflow::Visible),
                "hidden" => self.of_x = Some(Overflow::Hidden),
                "clip" => self.of_x = Some(Overflow::Clip),
                "scroll" => self.of_x = Some(Overflow::Scroll),
                _ => {}
            }
            return;
        }
        if let Some(suffix) = class.strip_prefix("overflow-y-") {
            match suffix {
                "visible" => self.of_y = Some(Overflow::Visible),
                "hidden" => self.of_y = Some(Overflow::Hidden),
                "clip" => self.of_y = Some(Overflow::Clip),
                "scroll" => self.of_y = Some(Overflow::Scroll),
                _ => {}
            }
            return;
        }
        if let Some(suffix) = class.strip_prefix("overflow-") {
            match suffix {
                "visible" => {
                    self.of_x = Some(Overflow::Visible);
                    self.of_y = Some(Overflow::Visible);
                }
                "hidden" => {
                    self.of_x = Some(Overflow::Hidden);
                    self.of_y = Some(Overflow::Hidden);
                }
                "clip" => {
                    self.of_x = Some(Overflow::Clip);
                    self.of_y = Some(Overflow::Clip);
                }
                "scroll" => {
                    self.of_x = Some(Overflow::Scroll);
                    self.of_y = Some(Overflow::Scroll);
                }
                _ => {}
            }
            return;
        }

        // === Grid Logic ===
        #[cfg(feature = "layout-grid")]
        {
            // Row/Col Start/End
            if let Some(suffix) = class.strip_prefix("row-start-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|s| s.parse::<i16>().ok())
                    .or_else(|| suffix.parse::<i16>().ok())
                {
                    self.row_start = Some(GridPlacement::from_line_index(v));
                    return;
                }
            }
            if let Some(suffix) = class.strip_prefix("row-end-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|s| s.parse::<i16>().ok())
                    .or_else(|| suffix.parse::<i16>().ok())
                {
                    self.row_end = Some(GridPlacement::from_line_index(v));
                    return;
                }
            }
            if let Some(suffix) = class.strip_prefix("col-start-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|s| s.parse::<i16>().ok())
                    .or_else(|| suffix.parse::<i16>().ok())
                {
                    self.col_start = Some(GridPlacement::from_line_index(v));
                    return;
                }
            }
            if let Some(suffix) = class.strip_prefix("col-end-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|s| s.parse::<i16>().ok())
                    .or_else(|| suffix.parse::<i16>().ok())
                {
                    self.col_end = Some(GridPlacement::from_line_index(v));
                    return;
                }
            }
            // Spans (set end to Span)
            if let Some(suffix) = class.strip_prefix("row-span-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|s| s.parse::<u16>().ok())
                    .or_else(|| suffix.parse::<u16>().ok())
                {
                    self.row_end = Some(GridPlacement::Span(v));
                    return;
                }
            }
            if let Some(suffix) = class.strip_prefix("col-span-") {
                if let Some(v) = extract_bracket_value(suffix)
                    .and_then(|s| s.parse::<u16>().ok())
                    .or_else(|| suffix.parse::<u16>().ok())
                {
                    self.col_end = Some(GridPlacement::Span(v));
                    return;
                }
            }
            // Auto Flow
            match class {
                "grid-flow-row" => {
                    self.updates.push((
                        LayoutKey::GridAutoFlow,
                        Layout::GridAutoFlow(GridAutoFlow::Row),
                    ));
                    return;
                }
                "grid-flow-col" => {
                    self.updates.push((
                        LayoutKey::GridAutoFlow,
                        Layout::GridAutoFlow(GridAutoFlow::Column),
                    ));
                    return;
                }
                "grid-flow-row-dense" => {
                    self.updates.push((
                        LayoutKey::GridAutoFlow,
                        Layout::GridAutoFlow(GridAutoFlow::RowDense),
                    ));
                    return;
                }
                "grid-flow-col-dense" => {
                    self.updates.push((
                        LayoutKey::GridAutoFlow,
                        Layout::GridAutoFlow(GridAutoFlow::ColumnDense),
                    ));
                    return;
                }
                _ => {}
            }
            // Justify Items
            match class {
                "justify-items-start" => {
                    self.updates.push((
                        LayoutKey::JustifyItems,
                        Layout::JustifyItems(AlignItems::Start),
                    ));
                    return;
                }
                "justify-items-end" => {
                    self.updates.push((
                        LayoutKey::JustifyItems,
                        Layout::JustifyItems(AlignItems::End),
                    ));
                    return;
                }
                "justify-items-center" => {
                    self.updates.push((
                        LayoutKey::JustifyItems,
                        Layout::JustifyItems(AlignItems::Center),
                    ));
                    return;
                }
                "justify-items-stretch" => {
                    self.updates.push((
                        LayoutKey::JustifyItems,
                        Layout::JustifyItems(AlignItems::Stretch),
                    ));
                    return;
                }
                _ => {}
            }
            // Justify Self
            match class {
                "justify-self-start" => {
                    self.updates.push((
                        LayoutKey::JustifySelf,
                        Layout::JustifySelf(AlignSelf::Start),
                    ));
                    return;
                }
                "justify-self-end" => {
                    self.updates
                        .push((LayoutKey::JustifySelf, Layout::JustifySelf(AlignSelf::End)));
                    return;
                }
                "justify-self-center" => {
                    self.updates.push((
                        LayoutKey::JustifySelf,
                        Layout::JustifySelf(AlignSelf::Center),
                    ));
                    return;
                }
                "justify-self-stretch" => {
                    self.updates.push((
                        LayoutKey::JustifySelf,
                        Layout::JustifySelf(AlignSelf::Stretch),
                    ));
                    return;
                }
                _ => {}
            }
        }

        // === Atomic Matches ===
        match class {
            #[cfg(feature = "layout-flex")]
            "flex" => self
                .updates
                .push((LayoutKey::Display, Layout::Display(Display::Flex))),
            #[cfg(feature = "layout-block")]
            "block" => self
                .updates
                .push((LayoutKey::Display, Layout::Display(Display::Block))),
            #[cfg(feature = "layout-grid")]
            "grid" => self
                .updates
                .push((LayoutKey::Display, Layout::Display(Display::Grid))),
            "hidden" => self
                .updates
                .push((LayoutKey::Display, Layout::Display(Display::None))),
            "box-border" => self.updates.push((
                LayoutKey::BoxSizing,
                Layout::BoxSizing(BoxSizing::BorderBox),
            )),
            "box-content" => self.updates.push((
                LayoutKey::BoxSizing,
                Layout::BoxSizing(BoxSizing::ContentBox),
            )),
            "relative" => self
                .updates
                .push((LayoutKey::Position, Layout::Position(Position::Relative))),
            "absolute" => self
                .updates
                .push((LayoutKey::Position, Layout::Position(Position::Absolute))),
            #[cfg(feature = "layout-flex")]
            "flex-row" => self.updates.push((
                LayoutKey::FlexDirection,
                Layout::FlexDirection(FlexDirection::Row),
            )),
            #[cfg(feature = "layout-flex")]
            "flex-row-reverse" => self.updates.push((
                LayoutKey::FlexDirection,
                Layout::FlexDirection(FlexDirection::RowReverse),
            )),
            #[cfg(feature = "layout-flex")]
            "flex-col" => self.updates.push((
                LayoutKey::FlexDirection,
                Layout::FlexDirection(FlexDirection::Column),
            )),
            #[cfg(feature = "layout-flex")]
            "flex-col-reverse" => self.updates.push((
                LayoutKey::FlexDirection,
                Layout::FlexDirection(FlexDirection::ColumnReverse),
            )),
            #[cfg(feature = "layout-flex")]
            "flex-wrap" => self
                .updates
                .push((LayoutKey::FlexWrap, Layout::FlexWrap(FlexWrap::Wrap))),
            #[cfg(feature = "layout-flex")]
            "flex-wrap-reverse" => self
                .updates
                .push((LayoutKey::FlexWrap, Layout::FlexWrap(FlexWrap::WrapReverse))),
            #[cfg(feature = "layout-flex")]
            "flex-nowrap" => self
                .updates
                .push((LayoutKey::FlexWrap, Layout::FlexWrap(FlexWrap::NoWrap))),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "items-start" => self
                .updates
                .push((LayoutKey::AlignItems, Layout::AlignItems(AlignItems::Start))),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "items-end" => self
                .updates
                .push((LayoutKey::AlignItems, Layout::AlignItems(AlignItems::End))),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "items-center" => self.updates.push((
                LayoutKey::AlignItems,
                Layout::AlignItems(AlignItems::Center),
            )),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "items-baseline" => self.updates.push((
                LayoutKey::AlignItems,
                Layout::AlignItems(AlignItems::Baseline),
            )),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "items-stretch" => self.updates.push((
                LayoutKey::AlignItems,
                Layout::AlignItems(AlignItems::Stretch),
            )),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "self-start" => self
                .updates
                .push((LayoutKey::AlignSelf, Layout::AlignSelf(AlignSelf::Start))),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "self-end" => self
                .updates
                .push((LayoutKey::AlignSelf, Layout::AlignSelf(AlignSelf::End))),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "self-center" => self
                .updates
                .push((LayoutKey::AlignSelf, Layout::AlignSelf(AlignSelf::Center))),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "self-baseline" => self
                .updates
                .push((LayoutKey::AlignSelf, Layout::AlignSelf(AlignSelf::Baseline))),
            #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
            "self-stretch" => self
                .updates
                .push((LayoutKey::AlignSelf, Layout::AlignSelf(AlignSelf::Stretch))),
            _ => {
                // Remaining prefixes
                #[cfg(feature = "layout-flex")]
                {
                    if let Some(suffix) = class.strip_prefix("grow-") {
                        if let Some(v) = extract_bracket_value(suffix)
                            .and_then(|s| s.parse::<f32>().ok())
                            .or_else(|| suffix.parse::<f32>().ok())
                        {
                            self.updates
                                .push((LayoutKey::FlexGrow, Layout::FlexGrow(v)));
                        } else if class == "grow" {
                            self.updates
                                .push((LayoutKey::FlexGrow, Layout::FlexGrow(1.0)));
                        }
                    }
                    if class == "grow" {
                        self.updates
                            .push((LayoutKey::FlexGrow, Layout::FlexGrow(1.0)));
                    }
                    if let Some(suffix) = class.strip_prefix("shrink-") {
                        if let Some(v) = extract_bracket_value(suffix)
                            .and_then(|s| s.parse::<f32>().ok())
                            .or_else(|| suffix.parse::<f32>().ok())
                        {
                            self.updates
                                .push((LayoutKey::FlexShrink, Layout::FlexShrink(v)));
                        } else if class == "shrink" {
                            self.updates
                                .push((LayoutKey::FlexShrink, Layout::FlexShrink(1.0)));
                        }
                    }
                    if class == "shrink" {
                        self.updates
                            .push((LayoutKey::FlexShrink, Layout::FlexShrink(1.0)));
                    }
                    if let Some(suffix) = class.strip_prefix("basis-") {
                        if let Some(v) = extract_bracket_value(suffix)
                            .and_then(|v| parse_dimension(v, cfg))
                            .or_else(|| parse_dimension(suffix, cfg))
                        {
                            self.updates
                                .push((LayoutKey::FlexBasis, Layout::FlexBasis(v)));
                        }
                    }
                }
                #[cfg(feature = "layout-block")]
                if class.starts_with("text-") {
                    match class {
                        "text-left" => self.updates.push((
                            LayoutKey::TextAlign,
                            Layout::TextAlign(TextAlign::LegacyLeft),
                        )),
                        "text-center" => self.updates.push((
                            LayoutKey::TextAlign,
                            Layout::TextAlign(TextAlign::LegacyCenter),
                        )),
                        "text-right" => self.updates.push((
                            LayoutKey::TextAlign,
                            Layout::TextAlign(TextAlign::LegacyRight),
                        )),
                        _ => {}
                    }
                }
                if let Some(suffix) = class.strip_prefix("aspect-") {
                    if suffix == "square" {
                        self.updates
                            .push((LayoutKey::AspectRatio, Layout::AspectRatio(1.0)));
                    } else if suffix == "video" {
                        self.updates
                            .push((LayoutKey::AspectRatio, Layout::AspectRatio(16.0 / 9.0)));
                    } else if let Some(v) = extract_bracket_value(suffix) {
                        if let Some(n) = v.parse::<f32>().ok() {
                            self.updates
                                .push((LayoutKey::AspectRatio, Layout::AspectRatio(n)));
                        } else if v.contains('/') {
                            let p: Vec<&str> = v.split('/').collect();
                            if p.len() == 2 {
                                if let (Ok(n), Ok(d)) = (p[0].parse::<f32>(), p[1].parse::<f32>()) {
                                    if d != 0.0 {
                                        self.updates.push((
                                            LayoutKey::AspectRatio,
                                            Layout::AspectRatio(n / d),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(suffix) = class.strip_prefix("scrollbar-") {
                    if let Some(v) = extract_bracket_value(suffix)
                        .and_then(|s| s.strip_suffix("px")?.parse::<f32>().ok())
                        .or_else(|| suffix.parse::<f32>().ok())
                    {
                        self.updates
                            .push((LayoutKey::ScrollbarWidth, Layout::ScrollbarWidth(v)));
                    }
                }
                #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
                {
                    match class {
                        "justify-start" => self.updates.push((
                            LayoutKey::JustifyContent,
                            Layout::JustifyContent(JustifyContent::Start),
                        )),
                        "justify-end" => self.updates.push((
                            LayoutKey::JustifyContent,
                            Layout::JustifyContent(JustifyContent::End),
                        )),
                        "justify-center" => self.updates.push((
                            LayoutKey::JustifyContent,
                            Layout::JustifyContent(JustifyContent::Center),
                        )),
                        "justify-between" => self.updates.push((
                            LayoutKey::JustifyContent,
                            Layout::JustifyContent(JustifyContent::SpaceBetween),
                        )),
                        "justify-around" => self.updates.push((
                            LayoutKey::JustifyContent,
                            Layout::JustifyContent(JustifyContent::SpaceAround),
                        )),
                        "justify-evenly" => self.updates.push((
                            LayoutKey::JustifyContent,
                            Layout::JustifyContent(JustifyContent::SpaceEvenly),
                        )),
                        "justify-stretch" => self.updates.push((
                            LayoutKey::JustifyContent,
                            Layout::JustifyContent(JustifyContent::Stretch),
                        )),
                        "content-start" => self.updates.push((
                            LayoutKey::AlignContent,
                            Layout::AlignContent(AlignContent::Start),
                        )),
                        "content-end" => self.updates.push((
                            LayoutKey::AlignContent,
                            Layout::AlignContent(AlignContent::End),
                        )),
                        "content-center" => self.updates.push((
                            LayoutKey::AlignContent,
                            Layout::AlignContent(AlignContent::Center),
                        )),
                        "content-between" => self.updates.push((
                            LayoutKey::AlignContent,
                            Layout::AlignContent(AlignContent::SpaceBetween),
                        )),
                        "content-around" => self.updates.push((
                            LayoutKey::AlignContent,
                            Layout::AlignContent(AlignContent::SpaceAround),
                        )),
                        "content-evenly" => self.updates.push((
                            LayoutKey::AlignContent,
                            Layout::AlignContent(AlignContent::SpaceEvenly),
                        )),
                        "content-stretch" => self.updates.push((
                            LayoutKey::AlignContent,
                            Layout::AlignContent(AlignContent::Stretch),
                        )),
                        _ => {}
                    }
                }
            }
        }
    }

    fn apply(self, selector: &mut LayoutStateSelector, state: ControlState) {
        for (key, layout) in self.updates {
            selector.update(state, key, layout);
        }

        if self.pl.is_some() || self.pr.is_some() || self.pt.is_some() || self.pb.is_some() {
            selector.update(
                state,
                LayoutKey::Padding,
                Layout::Padding(Rect {
                    left: self.pl.unwrap_or(LengthPercentage::Length(0.0)),
                    right: self.pr.unwrap_or(LengthPercentage::Length(0.0)),
                    top: self.pt.unwrap_or(LengthPercentage::Length(0.0)),
                    bottom: self.pb.unwrap_or(LengthPercentage::Length(0.0)),
                }),
            );
        }

        if self.ml.is_some() || self.mr.is_some() || self.mt.is_some() || self.mb.is_some() {
            selector.update(
                state,
                LayoutKey::Margin,
                Layout::Margin(Rect {
                    left: self.ml.unwrap_or(LengthPercentageAuto::Length(0.0)),
                    right: self.mr.unwrap_or(LengthPercentageAuto::Length(0.0)),
                    top: self.mt.unwrap_or(LengthPercentageAuto::Length(0.0)),
                    bottom: self.mb.unwrap_or(LengthPercentageAuto::Length(0.0)),
                }),
            );
        }

        if self.il.is_some() || self.ir.is_some() || self.it.is_some() || self.ib.is_some() {
            selector.update(
                state,
                LayoutKey::Inset,
                Layout::Inset(Rect {
                    left: self.il.unwrap_or(LengthPercentageAuto::Auto),
                    right: self.ir.unwrap_or(LengthPercentageAuto::Auto),
                    top: self.it.unwrap_or(LengthPercentageAuto::Auto),
                    bottom: self.ib.unwrap_or(LengthPercentageAuto::Auto),
                }),
            );
        }

        if self.w.is_some() || self.h.is_some() {
            selector.update(
                state,
                LayoutKey::Size,
                Layout::Size(Size {
                    width: self.w.unwrap_or(Dimension::Auto),
                    height: self.h.unwrap_or(Dimension::Auto),
                }),
            );
        }
        if self.min_w.is_some() || self.min_h.is_some() {
            selector.update(
                state,
                LayoutKey::MinSize,
                Layout::MinSize(Size {
                    width: self.min_w.unwrap_or(Dimension::Auto),
                    height: self.min_h.unwrap_or(Dimension::Auto),
                }),
            );
        }
        if self.max_w.is_some() || self.max_h.is_some() {
            selector.update(
                state,
                LayoutKey::MaxSize,
                Layout::MaxSize(Size {
                    width: self.max_w.unwrap_or(Dimension::Auto),
                    height: self.max_h.unwrap_or(Dimension::Auto),
                }),
            );
        }

        #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
        if self.gap_w.is_some() || self.gap_h.is_some() {
            selector.update(
                state,
                LayoutKey::Gap,
                Layout::Gap(Size {
                    width: self.gap_w.unwrap_or(LengthPercentage::Length(0.0)),
                    height: self.gap_h.unwrap_or(LengthPercentage::Length(0.0)),
                }),
            );
        }

        if self.of_x.is_some() || self.of_y.is_some() {
            selector.update(
                state,
                LayoutKey::Overflow,
                Layout::Overflow(Point {
                    x: self.of_x.unwrap_or(Overflow::Visible),
                    y: self.of_y.unwrap_or(Overflow::Visible),
                }),
            );
        }

        #[cfg(feature = "layout-grid")]
        {
            if self.row_start.is_some() || self.row_end.is_some() {
                selector.update(
                    state,
                    LayoutKey::GridRow,
                    Layout::GridRow(Line {
                        start: self.row_start.unwrap_or(GridPlacement::Auto),
                        end: self.row_end.unwrap_or(GridPlacement::Auto),
                    }),
                );
            }
            if self.col_start.is_some() || self.col_end.is_some() {
                selector.update(
                    state,
                    LayoutKey::GridColumn,
                    Layout::GridColumn(Line {
                        start: self.col_start.unwrap_or(GridPlacement::Auto),
                        end: self.col_end.unwrap_or(GridPlacement::Auto),
                    }),
                );
            }
        }
    }
}

/// Parse state prefix (hover:, focus:, active:, disabled:) from class name
fn parse_state_prefix(class: &str) -> (ControlState, &str) {
    if let Some(rest) = class.strip_prefix("hover:") {
        (ControlState::Hover, rest)
    } else if let Some(rest) = class.strip_prefix("focus:") {
        (ControlState::Focus, rest)
    } else if let Some(rest) = class.strip_prefix("active:") {
        (ControlState::Active, rest)
    } else if let Some(rest) = class.strip_prefix("disabled:") {
        (ControlState::Disable, rest)
    } else {
        (ControlState::Normal, class)
    }
}

/// State-aware accumulators for parsing classes with state prefixes
#[derive(Default)]
struct StateAccumulators {
    normal: LayoutAccumulator,
    hover: LayoutAccumulator,
    focus: LayoutAccumulator,
    active: LayoutAccumulator,
    disabled: LayoutAccumulator,
}

impl StateAccumulators {
    fn get_mut(&mut self, state: ControlState) -> &mut LayoutAccumulator {
        match state {
            ControlState::Normal => &mut self.normal,
            ControlState::Hover => &mut self.hover,
            ControlState::Focus => &mut self.focus,
            ControlState::Active => &mut self.active,
            ControlState::Disable => &mut self.disabled,
        }
    }

    fn apply_all(self, selector: &mut LayoutStateSelector) {
        self.normal.apply(selector, ControlState::Normal);
        self.hover.apply(selector, ControlState::Hover);
        self.focus.apply(selector, ControlState::Focus);
        self.active.apply(selector, ControlState::Active);
        self.disabled.apply(selector, ControlState::Disable);
    }
}

impl ClassLoader for LayoutStateSelector {
    fn load_classes(&mut self, class_str: &[&str]) {
        let cfg = UnitConfig::from_selector(self);
        let mut accs = StateAccumulators::default();
        for class in class_str {
            let (state, actual_class) = parse_state_prefix(class);
            accs.get_mut(state).parse(actual_class, &cfg);
        }
        accs.apply_all(self);
    }
}

