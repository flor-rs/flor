#[cfg(feature = "class")]
pub mod accumulators;
#[cfg(feature = "class")]
mod class;
#[cfg(feature = "class")]
pub use class::*;

use crate::view::resolver::{ResolverComputeMap, Unit, UnitResolver};
use crate::view::ViewId;
use flor_macros::Resolver;
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
    GridAutoFlow, GridPlacement, GridTrackRepetition, Line, MaxTrackSizingFunction,
    MinTrackSizingFunction, NonRepeatedTrackSizingFunction, TrackSizingFunction,
};

#[derive(Clone, Debug, Resolver)]
#[cfg_attr(feature = "devtools", resolver(serde_serialize = true))]
#[resolver(update_view = false,computed = false,data = Style,default = false,builder = false)]
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
    ScrollbarWidth(f32, Unit),

    // Position properties
    /// What should the `position` value of this struct use as a base offset?
    Position(Position),
    /// How should the position of this element be tweaked relative to the layout defined?
    Inset(Rect<LengthPercentageAuto>, Rect<Unit>),

    // Size properties
    /// Sets the initial size of the item
    Size(Size<Dimension>, Size<Unit>),
    /// Controls the minimum size of the item
    MinSize(Size<Dimension>, Size<Unit>),
    /// Controls the maximum size of the item
    MaxSize(Size<Dimension>, Size<Unit>),
    /// Sets the preferred aspect ratio for the item
    ///
    /// The ratio is calculated as width divided by height.
    AspectRatio(f32),

    // Spacing Properties
    /// How large should the margin be on each side?
    Margin(Rect<LengthPercentageAuto>, Rect<Unit>),
    /// How large should the padding be on each side?
    Padding(Rect<LengthPercentage>, Rect<Unit>),

    /// How large should the border be on each side?
    Border(Rect<LengthPercentage>, Rect<Unit>),

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
    Gap(Size<LengthPercentage>, Size<Unit>),

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
    FlexBasis(Dimension, Unit),
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
    GridTemplateRows(Vec<(TrackSizingFunction, Unit)>),
    /// Defines the track sizing functions (widths) of the grid columns
    #[cfg(feature = "layout-grid")]
    GridTemplateColumns(Vec<(TrackSizingFunction, Unit)>),
    /// Defines the size of implicitly created rows
    #[cfg(feature = "layout-grid")]
    GridAutoRows(Vec<(NonRepeatedTrackSizingFunction, Unit)>),
    /// Defined the size of implicitly created columns
    #[cfg(feature = "layout-grid")]
    GridAutoColumns(Vec<(NonRepeatedTrackSizingFunction, Unit)>),
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
    unit_resolver: &UnitResolver,
    state_variants: &ResolverComputeMap<LayoutKey, Layout>,
) -> Style {
    let mut layout_style = Style::DEFAULT;

    // 注意：这里必须用 iter() 获取 key，用于堆数据的查重
    for (_key, layout) in state_variants.iter() {
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
            Layout::ScrollbarWidth(v, unit) => {
                layout_style.scrollbar_width = unit_resolver.parse_unit(*v, *unit)
            }
            Layout::Position(v) => layout_style.position = *v,
            Layout::Inset(v, units) => {
                layout_style.inset = resolve_rect_lpa(unit_resolver, v, units)
            }
            Layout::Size(v, units) => layout_style.size = resolve_size_dim(unit_resolver, v, units),
            Layout::MinSize(v, units) => {
                layout_style.min_size = resolve_size_dim(unit_resolver, v, units)
            }
            Layout::MaxSize(v, units) => {
                layout_style.max_size = resolve_size_dim(unit_resolver, v, units)
            }
            Layout::AspectRatio(v) => layout_style.aspect_ratio = Some(*v),
            Layout::Margin(v, units) => {
                layout_style.margin = resolve_rect_lpa(unit_resolver, v, units)
            }
            Layout::Padding(v, units) => {
                layout_style.padding = resolve_rect_lp(unit_resolver, v, units)
            }
            Layout::Border(v, units) => {
                layout_style.border = resolve_rect_lp(unit_resolver, v, units)
            }

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
            Layout::Gap(v, units) => layout_style.gap = resolve_size_lp(unit_resolver, v, units),

            #[cfg(feature = "layout-block")]
            Layout::TextAlign(v) => layout_style.text_align = *v,

            #[cfg(feature = "layout-flex")]
            Layout::FlexDirection(v) => layout_style.flex_direction = *v,
            #[cfg(feature = "layout-flex")]
            Layout::FlexWrap(v) => layout_style.flex_wrap = *v,
            #[cfg(feature = "layout-flex")]
            Layout::FlexBasis(v, unit) => {
                layout_style.flex_basis = {
                    match v {
                        Dimension::Length(length) => {
                            Dimension::Length(unit_resolver.parse_unit(*length, *unit))
                        }
                        Dimension::Percent(percent) => Dimension::Percent(*percent),
                        Dimension::Auto => Dimension::Auto,
                    }
                }
            }
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
            // 说明：Resolver 已经在内部完成了状态合并，这里直接 clone
            // =========================================================
            #[cfg(feature = "layout-grid")]
            Layout::GridTemplateRows(v) => {
                layout_style.grid_template_rows = v
                    .iter()
                    .map(|(t, unit)| resolve_track_sizing(unit_resolver, t, unit))
                    .collect()
            }
            #[cfg(feature = "layout-grid")]
            Layout::GridTemplateColumns(v) => {
                layout_style.grid_template_columns = v
                    .iter()
                    .map(|(t, unit)| resolve_track_sizing(unit_resolver, t, unit))
                    .collect()
            }
            #[cfg(feature = "layout-grid")]
            Layout::GridAutoRows(v) => {
                layout_style.grid_auto_rows = v
                    .iter()
                    .map(|(t, unit)| resolve_non_repeated(unit_resolver, t, unit))
                    .collect()
            }
            #[cfg(feature = "layout-grid")]
            Layout::GridAutoColumns(v) => {
                layout_style.grid_auto_columns = v
                    .iter()
                    .map(|(t, unit)| resolve_non_repeated(unit_resolver, t, unit))
                    .collect()
            }
        }
    }

    layout_style
}

// =============================================================================
// 辅助转换函数：项目类型 (值 + 单位) → taffy 类型 (含 f32)
// =============================================================================

#[inline]
fn resolve_lp(ur: &UnitResolver, v: &LengthPercentage, unit: &Unit) -> LengthPercentage {
    match v {
        LengthPercentage::Length(length) => LengthPercentage::Length(ur.parse_unit(*length, *unit)),
        LengthPercentage::Percent(p) => LengthPercentage::Percent(*p),
    }
}

#[inline]
fn resolve_lpa(ur: &UnitResolver, v: &LengthPercentageAuto, unit: &Unit) -> LengthPercentageAuto {
    match v {
        LengthPercentageAuto::Length(length) => {
            LengthPercentageAuto::Length(ur.parse_unit(*length, *unit))
        }
        LengthPercentageAuto::Percent(p) => LengthPercentageAuto::Percent(*p),
        LengthPercentageAuto::Auto => LengthPercentageAuto::Auto,
    }
}

#[inline]
fn resolve_dim(ur: &UnitResolver, v: &Dimension, unit: &Unit) -> Dimension {
    match v {
        Dimension::Length(length) => Dimension::Length(ur.parse_unit(*length, *unit)),
        Dimension::Percent(p) => Dimension::Percent(*p),
        Dimension::Auto => Dimension::Auto,
    }
}

#[inline]
fn resolve_rect_lpa(
    ur: &UnitResolver,
    v: &Rect<LengthPercentageAuto>,
    units: &Rect<Unit>,
) -> Rect<LengthPercentageAuto> {
    Rect {
        left: resolve_lpa(ur, &v.left, &units.left),
        right: resolve_lpa(ur, &v.right, &units.right),
        top: resolve_lpa(ur, &v.top, &units.top),
        bottom: resolve_lpa(ur, &v.bottom, &units.bottom),
    }
}

#[inline]
fn resolve_rect_lp(
    ur: &UnitResolver,
    v: &Rect<LengthPercentage>,
    units: &Rect<Unit>,
) -> Rect<LengthPercentage> {
    Rect {
        left: resolve_lp(ur, &v.left, &units.left),
        right: resolve_lp(ur, &v.right, &units.right),
        top: resolve_lp(ur, &v.top, &units.top),
        bottom: resolve_lp(ur, &v.bottom, &units.bottom),
    }
}

#[inline]
fn resolve_size_dim(ur: &UnitResolver, v: &Size<Dimension>, units: &Size<Unit>) -> Size<Dimension> {
    Size {
        width: resolve_dim(ur, &v.width, &units.width),
        height: resolve_dim(ur, &v.height, &units.height),
    }
}

#[inline]
fn resolve_size_lp(
    ur: &UnitResolver,
    v: &Size<LengthPercentage>,
    units: &Size<Unit>,
) -> Size<LengthPercentage> {
    Size {
        width: resolve_lp(ur, &v.width, &units.width),
        height: resolve_lp(ur, &v.height, &units.height),
    }
}

#[cfg(feature = "layout-grid")]
fn resolve_min_track(
    ur: &UnitResolver,
    v: &MinTrackSizingFunction,
    unit: &Unit,
) -> MinTrackSizingFunction {
    match v {
        MinTrackSizingFunction::Fixed(lp) => {
            MinTrackSizingFunction::Fixed(resolve_lp(ur, lp, unit))
        }
        MinTrackSizingFunction::MinContent => MinTrackSizingFunction::MinContent,
        MinTrackSizingFunction::MaxContent => MinTrackSizingFunction::MaxContent,
        MinTrackSizingFunction::Auto => MinTrackSizingFunction::Auto,
    }
}

#[cfg(feature = "layout-grid")]
fn resolve_max_track(
    ur: &UnitResolver,
    v: &MaxTrackSizingFunction,
    unit: &Unit,
) -> MaxTrackSizingFunction {
    match v {
        MaxTrackSizingFunction::Fixed(lp) => {
            MaxTrackSizingFunction::Fixed(resolve_lp(ur, lp, unit))
        }
        MaxTrackSizingFunction::MinContent => MaxTrackSizingFunction::MinContent,
        MaxTrackSizingFunction::MaxContent => MaxTrackSizingFunction::MaxContent,
        MaxTrackSizingFunction::FitContent(lp) => {
            MaxTrackSizingFunction::FitContent(resolve_lp(ur, lp, unit))
        }
        MaxTrackSizingFunction::Auto => MaxTrackSizingFunction::Auto,
        MaxTrackSizingFunction::Fraction(f) => MaxTrackSizingFunction::Fraction(*f),
    }
}

#[cfg(feature = "layout-grid")]
fn resolve_non_repeated(
    ur: &UnitResolver,
    v: &NonRepeatedTrackSizingFunction,
    unit: &Unit,
) -> NonRepeatedTrackSizingFunction {
    NonRepeatedTrackSizingFunction {
        min: resolve_min_track(ur, &v.min, unit),
        max: resolve_max_track(ur, &v.max, unit),
    }
}

#[cfg(feature = "layout-grid")]
fn resolve_track_sizing(
    ur: &UnitResolver,
    v: &TrackSizingFunction,
    unit: &Unit,
) -> TrackSizingFunction {
    match v {
        TrackSizingFunction::Single(nr) => {
            TrackSizingFunction::Single(resolve_non_repeated(ur, nr, unit))
        }
        TrackSizingFunction::Repeat(rep, tracks) => {
            let taffy_rep = match rep {
                GridTrackRepetition::AutoFill => GridTrackRepetition::AutoFill,
                GridTrackRepetition::AutoFit => GridTrackRepetition::AutoFit,
                GridTrackRepetition::Count(n) => GridTrackRepetition::Count(*n),
            };
            TrackSizingFunction::Repeat(
                taffy_rep,
                tracks
                    .iter()
                    .map(|t| resolve_non_repeated(ur, t, unit))
                    .collect(),
            )
        }
    }
}
