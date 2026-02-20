use crate::view::control_state::ControlState;


use crate::view::resolver::layout::{Layout, LayoutKey, LayoutResolver};
use crate::view::resolver::shared::extract_bracket_value;

use crate::view::resolver::UnitResolver;
#[cfg(feature = "layout-grid")]
use taffy::style_helpers::TaffyGridLine;
#[cfg(feature = "layout-block")]
use taffy::TextAlign;
#[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
use taffy::{AlignContent, AlignItems, AlignSelf, JustifyContent};
use taffy::{
    BoxSizing, Dimension, Display, LengthPercentage, LengthPercentageAuto, Overflow, Point,
    Position, Rect, Size,
};
#[cfg(feature = "layout-flex")]
use taffy::{FlexDirection, FlexWrap};
#[cfg(feature = "layout-grid")]
use taffy::{
    GridAutoFlow, GridPlacement, Line, NonRepeatedTrackSizingFunction, TrackSizingFunction,
};

#[derive(Default)]
#[allow(dead_code)] // Fields may be unused depending on feature flags
pub(crate) struct LayoutAccumulator {
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
    pub(crate) fn parse(&mut self, class: &str, cfg: &UnitResolver) {
        let class = class.trim();
        if class.is_empty() {
            return;
        }

        // Specific prefixes first

        // === Padding (Specific) ===
        if let Some(suffix) = class.strip_prefix("pl-") {
            if let Some(v) = cfg.resolve_lp(suffix) {
                self.pl = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("pr-") {
            if let Some(v) = cfg.resolve_lp(suffix) {
                self.pr = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("pt-") {
            if let Some(v) = cfg.resolve_lp(suffix) {
                self.pt = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("pb-") {
            if let Some(v) = cfg.resolve_lp(suffix) {
                self.pb = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("px-") {
            if let Some(v) = cfg.resolve_lp(suffix) {
                self.pl = Some(v);
                self.pr = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("py-") {
            if let Some(v) = cfg.resolve_lp(suffix) {
                self.pt = Some(v);
                self.pb = Some(v);
                return;
            }
        }
        // === Padding (Generic) ===
        if let Some(suffix) = class.strip_prefix("p-") {
            if let Some(v) = cfg.resolve_lp(suffix) {
                self.pl = Some(v);
                self.pr = Some(v);
                self.pt = Some(v);
                self.pb = Some(v);
                return;
            }
        }

        // === Margin (Specific) ===
        if let Some(suffix) = class.strip_prefix("ml-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.ml = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("mr-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.mr = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("mt-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.mt = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("mb-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.mb = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("mx-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.ml = Some(v);
                self.mr = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("my-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.mt = Some(v);
                self.mb = Some(v);
                return;
            }
        }
        // === Margin (Generic) ===
        if let Some(suffix) = class.strip_prefix("m-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.ml = Some(v);
                self.mr = Some(v);
                self.mt = Some(v);
                self.mb = Some(v);
                return;
            }
        }

        // === Inset (Specific) ===
        if let Some(suffix) = class.strip_prefix("left-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.il = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("right-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.ir = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("top-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.it = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("bottom-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.ib = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("inset-x-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.il = Some(v);
                self.ir = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("inset-y-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.it = Some(v);
                self.ib = Some(v);
                return;
            }
        }
        // === Inset (Generic) ===
        if let Some(suffix) = class.strip_prefix("inset-") {
            if let Some(v) = cfg.resolve_lpa(suffix) {
                self.il = Some(v);
                self.ir = Some(v);
                self.it = Some(v);
                self.ib = Some(v);
                return;
            }
        }

        // === Sizing ===
        if let Some(suffix) = class.strip_prefix("w-") {
            if let Some(v) = cfg.resolve_dim(suffix) {
                self.w = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("h-") {
            if let Some(v) = cfg.resolve_dim(suffix) {
                self.h = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("min-w-") {
            if let Some(v) = cfg.resolve_dim(suffix) {
                self.min_w = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("min-h-") {
            if let Some(v) = cfg.resolve_dim(suffix) {
                self.min_h = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("max-w-") {
            if let Some(v) = cfg.resolve_dim(suffix) {
                self.max_w = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("max-h-") {
            if let Some(v) = cfg.resolve_dim(suffix) {
                self.max_h = Some(v);
                return;
            }
        }
        if let Some(suffix) = class.strip_prefix("size-") {
            if let Some(v) = cfg.resolve_dim(suffix) {
                self.w = Some(v);
                self.h = Some(v);
                return;
            }
        }

        // === Gap ===
        #[cfg(any(feature = "layout-flex", feature = "layout-grid"))]
        {
            if let Some(suffix) = class.strip_prefix("gap-x-") {
                if let Some(v) = cfg.resolve_lp(suffix) {
                    self.gap_w = Some(v);
                    return;
                }
            }
            if let Some(suffix) = class.strip_prefix("gap-y-") {
                if let Some(v) = cfg.resolve_lp(suffix) {
                    self.gap_h = Some(v);
                    return;
                }
            }
            if let Some(suffix) = class.strip_prefix("gap-") {
                if let Some(v) = cfg.resolve_lp(suffix) {
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
                        if let Some(v) = cfg.resolve_dim(suffix) {
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

    pub(crate) fn apply(self, selector: &mut LayoutResolver, state: ControlState) {
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
