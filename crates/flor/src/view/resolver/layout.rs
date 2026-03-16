#[cfg(feature = "class")]
pub mod accumulators;
#[cfg(feature = "class")]
mod class;
#[cfg(feature = "class")]
pub use class::*;

use crate::view::control_state::ControlState;
use crate::view::resolver::UnitResolver;
use crate::view::ViewId;
use flor_macros::Resolver;
use rustc_hash::FxHashMap;
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

#[derive(Clone, Debug, Resolver)]
#[resolver(update_view = false,computed = false,data = taffy::Style,default = false,builder = false
)]
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

impl LayoutResolver {
    #[inline]
    pub fn new(view_id: ViewId) -> Self {
        Self::new_with_compute_func(view_id, computed_layout)
    }
}

fn computed_layout(
    _unit_resolver: &UnitResolver,
    state: ControlState,
    state_variants: &FxHashMap<ControlState, FxHashMap<LayoutKey, Layout>>,
) -> Style {
    let mut layout_style = Style::default();

    // 1. 预先获取特定状态（Specific）的 Map 引用
    // 用于在处理 Normal 层的昂贵数据时进行“查重”，避免无效 Clone
    let specific_variants = if state == ControlState::Normal {
        None
    } else {
        state_variants.get(&state)
    };

    // 2. 应用 Normal (Base) 层
    if let Some(normal_map) = state_variants.get(&ControlState::Normal) {
        // 注意：这里必须用 iter() 获取 key，用于堆数据的查重
        for (_key, layout) in normal_map.iter() {
            match layout {
                // =========================================================
                // 场景 A：栈上数据 (Copy Types)
                // 策略：直接赋值 (Blind Write)
                // 原因：MOV 指令成本远低于 HashMap 哈希查找成本，无需检查
                // =========================================================
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
                Layout::GridAutoFlow(v) => layout_style.grid_auto_flow = *v,
                #[cfg(feature = "layout-grid")]
                Layout::GridRow(v) => layout_style.grid_row = *v,
                #[cfg(feature = "layout-grid")]
                Layout::GridColumn(v) => layout_style.grid_column = *v,

                // =========================================================
                // 场景 B：堆上数据 (Heap Types / Clone)
                // 策略：防御性检查 (Check before Clone)
                // 原因：HashMap 查找成本远低于 malloc/free 成本
                // =========================================================
                #[cfg(feature = "layout-grid")]
                Layout::GridTemplateRows(v) => {
                    // 如果 Specific 层没有定义这个 Key，才从 Normal 层 Clone
                    if specific_variants.map_or(true, |s| !s.contains_key(key)) {
                        layout_style.grid_template_rows = v.clone();
                    }
                }
                #[cfg(feature = "layout-grid")]
                Layout::GridTemplateColumns(v) => {
                    if specific_variants.map_or(true, |s| !s.contains_key(key)) {
                        layout_style.grid_template_columns = v.clone();
                    }
                }
                #[cfg(feature = "layout-grid")]
                Layout::GridAutoRows(v) => {
                    if specific_variants.map_or(true, |s| !s.contains_key(key)) {
                        layout_style.grid_auto_rows = v.clone();
                    }
                }
                #[cfg(feature = "layout-grid")]
                Layout::GridAutoColumns(v) => {
                    if specific_variants.map_or(true, |s| !s.contains_key(key)) {
                        layout_style.grid_auto_columns = v.clone();
                    }
                }
            }
        }
    }

    // 3. 应用 Specific (Current State) 层
    // 逻辑：如果当前存在特定状态数据，无条件覆盖（这是正确的 CSS 层叠逻辑）
    if let Some(map) = specific_variants {
        for layout in map.values() {
            match layout {
                // Copy Types: 直接覆盖
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
                TextAlign(v) => layout_style.text_align = *v,

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
                Layout::GridAutoFlow(v) => layout_style.grid_auto_flow = *v,
                #[cfg(feature = "layout-grid")]
                Layout::GridRow(v) => layout_style.grid_row = *v,
                #[cfg(feature = "layout-grid")]
                Layout::GridColumn(v) => layout_style.grid_column = *v,

                // Clone Types: 必须覆盖，无法避免 Clone
                #[cfg(feature = "layout-grid")]
                Layout::GridTemplateRows(v) => layout_style.grid_template_rows = v.clone(),
                #[cfg(feature = "layout-grid")]
                Layout::GridTemplateColumns(v) => layout_style.grid_template_columns = v.clone(),
                #[cfg(feature = "layout-grid")]
                Layout::GridAutoRows(v) => layout_style.grid_auto_rows = v.clone(),
                #[cfg(feature = "layout-grid")]
                Layout::GridAutoColumns(v) => layout_style.grid_auto_columns = v.clone(),
            }
        }
    }

    layout_style
}
