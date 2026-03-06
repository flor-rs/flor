use crate::renderer::FONT_SYSTEM;
use cosmic_text::fontdb::ID;
use flor_base::graphics::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextFormatHandle,
    TextTrimming, WordWrapping,
};

#[derive(Debug, Clone)]
pub struct GlTextFormatHandle {
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
    pub custom_font_id: Option<ID>,
}

impl GlTextFormatHandle {
    pub fn new() -> Self {
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
            custom_font_id: None,
        }
    }
}

impl Drop for GlTextFormatHandle {
    fn drop(&mut self) {
        if let Some(id) = self.custom_font_id {
            if let Some(mut font_system_lock) = FONT_SYSTEM.get().map(|f| f.lock()) {
                font_system_lock.db_mut().remove_face(id);
            }
        }
    }
}

impl TextFormatHandle for GlTextFormatHandle {
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
}

impl GlTextFormatHandle {
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

    pub(crate) fn apply_wrap(
        &self,
        font_system: &mut cosmic_text::FontSystem,
        buffer: &mut cosmic_text::Buffer,
    ) {
        let wrap = if self.text_alignment == TextAlignment::Justified
            && self.to_cosmic_wrap() == cosmic_text::Wrap::None
        {
            cosmic_text::Wrap::Word
        } else {
            self.to_cosmic_wrap()
        };
        buffer.set_wrap(font_system, wrap);
    }

    pub(crate) fn apply_text(
        &self,
        font_system: &mut cosmic_text::FontSystem,
        buffer: &mut cosmic_text::Buffer,
        text: &str,
    ) {
        buffer.set_text(
            font_system,
            text,
            &self.to_cosmic_attrs(),
            cosmic_text::Shaping::Advanced,
            self.to_cosmic_align(),
        );
        buffer.shape_until_scroll(font_system, false);
    }

    pub(crate) fn create_buffer(
        &self,
        font_system: &mut cosmic_text::FontSystem,
        font_size: f32,
    ) -> cosmic_text::Buffer {
        cosmic_text::Buffer::new(
            font_system,
            cosmic_text::Metrics::new(font_size, font_size * self.line_height_factor),
        )
    }

    pub(crate) fn apply_text_with_trimming(
        &self,
        font_system: &mut cosmic_text::FontSystem,
        buffer: &mut cosmic_text::Buffer,
        text: &str,
        width: f32,
        height: f32,
        phys_width: f32,
        phys_height: f32,
    ) {
        if width <= 0.0 || self.text_trimming == TextTrimming::None {
            self.apply_text(font_system, buffer, text);
            return;
        }

        self.apply_text(font_system, buffer, text);

        let is_wrap = self.word_wrapping != WordWrapping::NoWrap;
        let is_word = self.text_trimming == TextTrimming::EllipsisWord;
        let has_ellipsis = self.text_trimming == TextTrimming::EllipsisChar || is_word;

        if is_wrap
            && (height <= 0.0
                || buffer.layout_runs().map(|r| r.line_height).sum::<f32>() <= phys_height)
        {
            return;
        }

        let mut overflow_found = false;
        let mut cutoff_idx = text.len();

        let mut single_dot_width = 0.0;
        let mut ellipsis_width = 0.0;

        if has_ellipsis {
            let mut test_buffer = self.create_buffer(font_system, self.font_size);

            self.apply_text(font_system, &mut test_buffer, ".");
            if let Some(run) = test_buffer.layout_runs().next() {
                single_dot_width = run.line_w;
            }

            self.apply_text(font_system, &mut test_buffer, "...");
            if let Some(run) = test_buffer.layout_runs().next() {
                ellipsis_width = run.line_w;
            }
        }

        let mut total_h = 0.0f32;
        let mut prev_line_valid_cutoff = 0;

        let target_w = phys_width - ellipsis_width;

        for run in buffer.layout_runs() {
            if height > 0.0 && total_h + run.line_height > phys_height {
                if total_h == 0.0 {
                    cutoff_idx = 0;
                } else {
                    cutoff_idx = prev_line_valid_cutoff;
                }
                overflow_found = true;
                break;
            }

            let last_fit_index = if let Some(last_fit) = run
                .glyphs
                .iter()
                .take_while(|g| g.x + g.w <= target_w)
                .last()
            {
                last_fit.end
            } else {
                run.glyphs.first().map(|g| g.start).unwrap_or(0)
            };

            if run.line_w > phys_width {
                cutoff_idx = last_fit_index;
                overflow_found = true;
                break;
            }

            prev_line_valid_cutoff = last_fit_index;

            total_h += run.line_height;
        }

        if overflow_found {
            let max_bytes = text.len().min(cutoff_idx);
            let mut test_text = text[..max_bytes].to_string();

            if is_word {
                if let Some(idx) = test_text.rfind(char::is_whitespace) {
                    test_text.truncate(idx);
                }
            }

            if has_ellipsis {
                if test_text.is_empty() && single_dot_width > 0.0 {
                    let dots_count = (phys_width / single_dot_width).floor() as usize;
                    test_text = ".".repeat(dots_count);
                } else {
                    test_text.push_str("...");
                }
            }

            self.apply_text(font_system, buffer, &test_text);
        }
    }

    pub(crate) fn calc_offset_y(&self, buffer: &cosmic_text::Buffer, phys_height: f32) -> f32 {
        let total_h: f32 = buffer.layout_runs().map(|r| r.line_height).sum();

        match self.paragraph_alignment {
            ParagraphAlignment::Center if phys_height > total_h => (phys_height - total_h) / 2.0,
            ParagraphAlignment::Bottom if phys_height > total_h => phys_height - total_h,
            _ => 0.0,
        }
    }
}
