use crate::graphics::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextTrimming,
    WordWrapping,
};
use cosmic_text::{Align, Attrs, Buffer, Family, Stretch, Style, Weight, Wrap};

#[derive(Debug, Clone)]
pub struct TextLayoutConfig {
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub font_stretch: FontStretch,
    pub font_family_name: String,
    pub text_alignment: TextAlignment,
    pub paragraph_alignment: ParagraphAlignment,
    pub word_wrapping: WordWrapping,
    pub line_height_factor: f32,
    pub text_trimming: TextTrimming,
}

impl Default for TextLayoutConfig {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            font_stretch: FontStretch::Normal,
            font_family_name: String::new(),
            text_alignment: TextAlignment::Start,
            paragraph_alignment: ParagraphAlignment::Top,
            word_wrapping: WordWrapping::NoWrap,
            line_height_factor: 1.0,
            text_trimming: TextTrimming::None,
        }
    }
}

impl TextLayoutConfig {
    pub fn to_cosmic_attrs(&self) -> Attrs<'_> {
        let weight = match self.font_weight {
            FontWeight::Thin => Weight::THIN,
            FontWeight::ExtraLight => Weight::EXTRA_LIGHT,
            FontWeight::Light => Weight::LIGHT,
            FontWeight::Normal => Weight::NORMAL,
            FontWeight::Medium => Weight::MEDIUM,
            FontWeight::SemiBold => Weight::SEMIBOLD,
            FontWeight::Bold => Weight::BOLD,
            FontWeight::ExtraBold => Weight::EXTRA_BOLD,
            FontWeight::Black => Weight::BLACK,
            FontWeight::ExtraBlack => Weight(950),
        };
        let style = match self.font_style {
            FontStyle::Normal => Style::Normal,
            FontStyle::Italic => Style::Italic,
            FontStyle::Oblique => Style::Oblique,
        };
        let stretch = match self.font_stretch {
            FontStretch::UltraCondensed => Stretch::UltraCondensed,
            FontStretch::ExtraCondensed => Stretch::ExtraCondensed,
            FontStretch::Condensed => Stretch::Condensed,
            FontStretch::SemiCondensed => Stretch::SemiCondensed,
            FontStretch::Normal => Stretch::Normal,
            FontStretch::SemiExpanded => Stretch::SemiExpanded,
            FontStretch::Expanded => Stretch::Expanded,
            FontStretch::ExtraExpanded => Stretch::ExtraExpanded,
            FontStretch::UltraExpanded => Stretch::UltraExpanded,
        };
        Attrs::new()
            .family(Family::Name(&self.font_family_name))
            .weight(weight)
            .style(style)
            .stretch(stretch)
    }

    pub fn to_cosmic_align(&self) -> Option<Align> {
        match self.text_alignment {
            TextAlignment::Start => Some(Align::Left),
            TextAlignment::Center => Some(Align::Center),
            TextAlignment::End => Some(Align::Right),
            TextAlignment::Justified => Some(Align::Justified),
        }
    }

    pub fn to_cosmic_wrap(&self) -> Wrap {
        match self.word_wrapping {
            WordWrapping::NoWrap => Wrap::None,
            WordWrapping::Wrap => Wrap::Word,
            WordWrapping::Character => Wrap::Glyph,
        }
    }

    pub fn calc_offset_y(&self, buffer: &Buffer, phys_height: f32) -> f32 {
        let total_h: f32 = buffer.layout_runs().map(|r| r.line_height).sum();

        match self.paragraph_alignment {
            ParagraphAlignment::Center if phys_height > total_h => (phys_height - total_h) / 2.0,
            ParagraphAlignment::Bottom if phys_height > total_h => phys_height - total_h,
            _ => 0.0,
        }
    }
}
