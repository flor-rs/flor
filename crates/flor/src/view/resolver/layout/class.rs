use crate::view::control_state::ControlState;
use crate::view::resolver::layout::accumulators::LayoutAccumulator;
use crate::view::resolver::shared::parse_state_prefix;
use crate::view::resolver::LayoutResolver;

pub trait Class {
    fn apply_layout(&self, layout_resolver: &mut LayoutResolver);
}

impl Class for Vec<String> {
    fn apply_layout(&self, layout_resolver: &mut LayoutResolver) {
        let _ = layout_resolver.unit_resolver.sync_unit();

        let mut normal = LayoutAccumulator::default();
        let mut hover = LayoutAccumulator::default();
        let mut focus = LayoutAccumulator::default();
        let mut active = LayoutAccumulator::default();
        let mut disabled = LayoutAccumulator::default();

        for class in self {
            let (state, actual_class) = parse_state_prefix(class);
            match state {
                ControlState::Normal => normal.parse(actual_class, &layout_resolver.unit_resolver),
                ControlState::Hover => hover.parse(actual_class, &layout_resolver.unit_resolver),
                ControlState::Focus => focus.parse(actual_class, &layout_resolver.unit_resolver),
                ControlState::Active => active.parse(actual_class, &layout_resolver.unit_resolver),
                ControlState::Disabled => {
                    disabled.parse(actual_class, &layout_resolver.unit_resolver)
                }
            }
        }

        normal.apply(layout_resolver, ControlState::Normal);
        hover.apply(layout_resolver, ControlState::Hover);
        focus.apply(layout_resolver, ControlState::Focus);
        active.apply(layout_resolver, ControlState::Active);
        disabled.apply(layout_resolver, ControlState::Disabled);
    }
}
