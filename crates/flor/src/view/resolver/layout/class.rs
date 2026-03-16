use crate::view::control_state::ControlState;
use crate::view::resolver::layout::accumulators::LayoutAccumulator;
use crate::view::resolver::shared::parse_state_prefix;
use crate::view::resolver::{LayoutResolver, UnitResolver};
use crate::view::ViewId;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct Class {
    pub class: Vec<String>,
    pub has_update: AtomicBool,
    pub unit_resolver: UnitResolver,
}

impl Class {
    pub fn new(view_id: ViewId) -> Self {
        Self {
            class: Vec::new(),
            has_update: AtomicBool::new(true),
            unit_resolver: UnitResolver::new(view_id),
        }
    }

    pub fn apply_layout(&self, layout_resolver: &mut LayoutResolver) {
        let just_synced = self.unit_resolver.sync_unit();
        if !self.has_update.load(Ordering::Acquire) && !just_synced {
            return;
        }
        let mut normal = LayoutAccumulator::default();
        let mut hover = LayoutAccumulator::default();
        let mut focus = LayoutAccumulator::default();
        let mut active = LayoutAccumulator::default();
        let mut disabled = LayoutAccumulator::default();

        for class in &self.class {
            let (state, actual_class) = parse_state_prefix(class);
            match state {
                ControlState::Normal => normal.parse(actual_class, &self.unit_resolver),
                ControlState::Hover => hover.parse(actual_class, &self.unit_resolver),
                ControlState::Focus => focus.parse(actual_class, &self.unit_resolver),
                ControlState::Active => active.parse(actual_class, &self.unit_resolver),
                ControlState::Disabled => disabled.parse(actual_class, &self.unit_resolver),
            }
        }
        layout_resolver.class_state_variants.clear();
        normal.apply(layout_resolver, ControlState::Normal);
        hover.apply(layout_resolver, ControlState::Hover);
        focus.apply(layout_resolver, ControlState::Focus);
        active.apply(layout_resolver, ControlState::Active);
        disabled.apply(layout_resolver, ControlState::Disabled);
        self.has_update.store(false, Ordering::Release);
    }

    pub fn set_update(&mut self) {
        self.has_update.store(true, Ordering::Release);
    }

    pub fn load_classes(&mut self, classes: Vec<String>) {
        self.class = classes;
        self.has_update.store(true, Ordering::Release);
    }
}
