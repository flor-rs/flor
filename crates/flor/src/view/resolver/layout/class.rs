use crate::view::class::ClassLoader;
use crate::view::control_state::ControlState;
use crate::view::resolver::layout::accumulators::LayoutAccumulator;
use crate::view::resolver::layout::LayoutResolver;
use crate::view::resolver::shared::parse_state_prefix;

impl ClassLoader for LayoutResolver {
    fn load_classes(&mut self, class_str: &[&str]) {
        let mut normal = LayoutAccumulator::default();
        let mut hover = LayoutAccumulator::default();
        let mut focus = LayoutAccumulator::default();
        let mut active = LayoutAccumulator::default();
        let mut disabled = LayoutAccumulator::default();

        for &class in class_str {
            let (state, actual_class) = parse_state_prefix(class);
            match state {
                ControlState::Normal => normal.parse(actual_class, &self.unit_resolver),
                ControlState::Hover => hover.parse(actual_class, &self.unit_resolver),
                ControlState::Focus => focus.parse(actual_class, &self.unit_resolver),
                ControlState::Active => active.parse(actual_class, &self.unit_resolver),
                ControlState::Disabled => disabled.parse(actual_class, &self.unit_resolver),
            }
        }

        normal.apply(self, ControlState::Normal);
        hover.apply(self, ControlState::Hover);
        focus.apply(self, ControlState::Focus);
        active.apply(self, ControlState::Active);
        disabled.apply(self, ControlState::Disabled);
    }
}
