use flor_base::graphics::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextFormatHandle,
    TextTrimming, WordWrapping,
};

#[derive(Debug, Clone)]
pub struct GlTextFormatHandle {}
impl TextFormatHandle for GlTextFormatHandle {
    fn set_font_size(&mut self, size: f32) -> &mut Self {
        self
    }

    fn font_size(&self) -> f32 {
        0.
    }

    fn set_font_weight(&mut self, weight: FontWeight) -> &mut Self {
        self
    }

    fn font_weight(&self) -> FontWeight {
        FontWeight::Normal
    }

    fn set_font_style(&mut self, style: FontStyle) -> &mut Self {
        self
    }

    fn font_style(&self) -> FontStyle {
        FontStyle::Normal
    }

    fn set_font_stretch(&mut self, stretch: FontStretch) -> &mut Self {
        self
    }

    fn font_stretch(&self) -> FontStretch {
        FontStretch::Normal
    }

    fn font_family_name(&self) -> String {
        String::new()
    }

    fn set_text_alignment(&mut self, align: TextAlignment) -> &mut Self {
        self
    }

    fn text_alignment(&self) -> TextAlignment {
        TextAlignment::End
    }

    fn set_paragraph_alignment(&mut self, align: ParagraphAlignment) -> &mut Self {
        self
    }

    fn paragraph_alignment(&self) -> ParagraphAlignment {
        ParagraphAlignment::Top
    }

    fn set_word_wrapping(&mut self, wrapping: WordWrapping) -> &mut Self {
        self
    }

    fn word_wrapping(&self) -> WordWrapping {
        WordWrapping::Character
    }

    fn set_line_height(&mut self, line_height_factor: f32) -> &mut Self {
        self
    }

    fn line_height(&self) -> f32 {
        0.
    }

    fn set_text_trimming(&mut self, trimming: TextTrimming) -> &mut Self {
        self
    }

    fn text_trimming(&self) -> TextTrimming {
        TextTrimming::Character
    }

    fn dirty(&self) -> bool {
        false
    }

    fn clear_dirty(&mut self) {}
}
