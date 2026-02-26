use flor_base::graphics::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextFormatHandle,
    TextTrimming, WordWrapping,
};
use slotmap::new_key_type;

#[derive(Debug, Clone)]
pub struct TinySkiaTextFormatHandle {
    pub font_system: std::sync::Arc<parking_lot::RwLock<cosmic_text::FontSystem>>,
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

impl TinySkiaTextFormatHandle {
    pub fn new(font_system: std::sync::Arc<parking_lot::RwLock<cosmic_text::FontSystem>>) -> Self {
        Self {
            font_system,
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

impl TextFormatHandle for TinySkiaTextFormatHandle {
    fn set_font_size(&mut self, size: f32) -> &mut Self {
        self.font_size = size;
        self
    }

    fn font_size(&self) -> f32 {
        self.font_size
    }

    fn set_font_weight(&mut self, weight: FontWeight) -> &mut Self {
        self.font_weight = weight;
        self
    }

    fn font_weight(&self) -> FontWeight {
        self.font_weight
    }

    fn set_font_style(&mut self, style: FontStyle) -> &mut Self {
        self.font_style = style;
        self
    }

    fn font_style(&self) -> FontStyle {
        self.font_style
    }

    fn set_font_stretch(&mut self, stretch: FontStretch) -> &mut Self {
        self.font_stretch = stretch;
        self
    }

    fn font_stretch(&self) -> FontStretch {
        self.font_stretch
    }

    fn font_family_name(&self) -> String {
        self.font_family_name.clone()
    }

    fn set_text_alignment(&mut self, align: TextAlignment) -> &mut Self {
        self.text_alignment = align;
        self
    }

    fn text_alignment(&self) -> TextAlignment {
        self.text_alignment
    }

    fn set_paragraph_alignment(&mut self, align: ParagraphAlignment) -> &mut Self {
        self.paragraph_alignment = align;
        self
    }

    fn paragraph_alignment(&self) -> ParagraphAlignment {
        self.paragraph_alignment
    }

    fn set_word_wrapping(&mut self, wrapping: WordWrapping) -> &mut Self {
        self.word_wrapping = wrapping;
        self
    }

    fn word_wrapping(&self) -> WordWrapping {
        self.word_wrapping
    }

    fn set_line_height(&mut self, line_height_factor: f32) -> &mut Self {
        self.line_height_factor = line_height_factor;
        self
    }

    fn line_height(&self) -> f32 {
        self.line_height_factor
    }

    fn set_text_trimming(&mut self, trimming: TextTrimming) -> &mut Self {
        self.text_trimming = trimming;
        self
    }

    fn text_trimming(&self) -> TextTrimming {
        self.text_trimming
    }

    fn dirty(&self) -> bool {
        false
    }

    fn clear_dirty(&mut self) {}
}

impl TinySkiaTextFormatHandle {
    pub(crate) fn to_cosmic_attrs(&'_ self) -> cosmic_text::Attrs<'_> {
        let weight = match self.font_weight {
            FontWeight::Thin => cosmic_text::Weight::THIN,
            FontWeight::ExtraLight => cosmic_text::Weight::EXTRA_LIGHT,
            FontWeight::Light => cosmic_text::Weight::LIGHT,
            FontWeight::Normal => cosmic_text::Weight::NORMAL,
            FontWeight::Medium => cosmic_text::Weight::MEDIUM,
            FontWeight::SemiBold => cosmic_text::Weight::SEMIBOLD,
            FontWeight::Bold => cosmic_text::Weight::BOLD,
            FontWeight::ExtraBold => cosmic_text::Weight::EXTRA_BOLD,
            FontWeight::Black => cosmic_text::Weight::BLACK,
            FontWeight::ExtraBlack => cosmic_text::Weight(950),
        };
        let style = match self.font_style {
            FontStyle::Normal => cosmic_text::Style::Normal,
            FontStyle::Italic => cosmic_text::Style::Italic,
            FontStyle::Oblique => cosmic_text::Style::Oblique,
        };
        let stretch = match self.font_stretch {
            FontStretch::UltraCondensed => cosmic_text::Stretch::UltraCondensed,
            FontStretch::ExtraCondensed => cosmic_text::Stretch::ExtraCondensed,
            FontStretch::Condensed => cosmic_text::Stretch::Condensed,
            FontStretch::SemiCondensed => cosmic_text::Stretch::SemiCondensed,
            FontStretch::Normal => cosmic_text::Stretch::Normal,
            FontStretch::SemiExpanded => cosmic_text::Stretch::SemiExpanded,
            FontStretch::Expanded => cosmic_text::Stretch::Expanded,
            FontStretch::ExtraExpanded => cosmic_text::Stretch::ExtraExpanded,
            FontStretch::UltraExpanded => cosmic_text::Stretch::UltraExpanded,
        };
        cosmic_text::Attrs::new()
            .family(cosmic_text::Family::Name(&self.font_family_name))
            .weight(weight)
            .style(style)
            .stretch(stretch)
    }

    pub(crate) fn to_cosmic_align(&self) -> Option<cosmic_text::Align> {
        match self.text_alignment {
            TextAlignment::Start => Some(cosmic_text::Align::Left),
            TextAlignment::Center => Some(cosmic_text::Align::Center),
            TextAlignment::End => Some(cosmic_text::Align::Right),
            TextAlignment::Justified => Some(cosmic_text::Align::Justified),
        }
    }

    pub(crate) fn to_cosmic_wrap(&self) -> cosmic_text::Wrap {
        match self.word_wrapping {
            WordWrapping::NoWrap => cosmic_text::Wrap::None,
            WordWrapping::Wrap => cosmic_text::Wrap::Word,
            WordWrapping::Character => cosmic_text::Wrap::Glyph,
        }
    }
}

new_key_type! {
    pub struct TextFormatSlotId;
}
