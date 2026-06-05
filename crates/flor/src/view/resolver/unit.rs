mod metrics;
mod resolver;

pub use {metrics::*, resolver::*};

    pub fn parse_length_percentage(&self, value: &str) -> Option<LengthPercentage> {
        if value == "full" {
            return Some(LengthPercentage::Percent(100.0));
        }
        if let Some(pct) = Self::parse_percent(value) {
            return Some(LengthPercentage::Percent(pct));
        }
        if let Some(pct) = Self::parse_fraction(value) {
            return Some(LengthPercentage::Percent(pct));
        }
        if let Some(px) = self.parse_unit_px(value) {
            return Some(LengthPercentage::Length(px));
        }
        None
    }

    Rem,

    Vw,
    Vh,
}

    #[inline]
    pub fn resolve_dim(&self, suffix: &str) -> Option<Dimension> {
        extract_bracket_value(suffix)
            .and_then(|v| self.parse_dimension(v))
            .or_else(|| self.parse_dimension(suffix))
    }

    #[inline]
    pub fn resolve_lp(&self, suffix: &str) -> Option<LengthPercentage> {
        extract_bracket_value(suffix)
            .and_then(|v| self.parse_length_percentage(v))
            .or_else(|| self.parse_length_percentage(suffix))
    }

    Rem(f32),

    Vw(f32),
    Vh(f32),
}
