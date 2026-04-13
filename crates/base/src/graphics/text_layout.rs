use crate::graphics::{
    FontStretch, FontStyle, FontWeight, HitTestResult, ParagraphAlignment, TextAlignment,
    TextTrimming, WordWrapping,
};
use cosmic_text::{
    Align, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Stretch, Style, Weight, Wrap,
};

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

/// Check whether the laid-out text overflows the given physical dimensions.
/// Returns `true` if any layout run is wider than `phys_width` or if the
/// accumulated line height exceeds `phys_height` (when height > 0).
fn check_layout_overflow(buffer: &Buffer, phys_width: f32, phys_height: f32, height: f32) -> bool {
    let mut total_h = 0.0f32;
    for run in buffer.layout_runs() {
        if run.line_w > phys_width {
            return true;
        }
        if height > 0.0 && total_h + run.line_height > phys_height {
            return true;
        }
        total_h += run.line_height;
    }
    false
}

/// Set text to buffer with appropriate method based on whether chunks are provided
fn set_text_to_buffer(
    buffer: &mut Buffer,
    font_system: &mut FontSystem,
    text: &str,
    chunks: Option<&[(Attrs, usize, usize)]>,
    config: &TextLayoutConfig,
) {
    if let Some(chunks) = chunks {
        if !chunks.is_empty() {
            let mut spans = Vec::new();
            let text_len = text.len();
            let mut last_end = 0;
            let default_attrs = config.to_cosmic_attrs();

            for chunk in chunks {
                let start = chunk.1.min(text_len);
                let end = (chunk.1 + chunk.2).min(text_len);

                if start > last_end {
                    spans.push((&text[last_end..start], default_attrs.clone()));
                }
                if end > start {
                    spans.push((&text[start..end], chunk.0.clone()));
                    last_end = end;
                }
            }

            // 补全最后一段
            if last_end < text_len {
                spans.push((&text[last_end..text_len], default_attrs.clone()));
            }

            // 使用默认属性作为基础，spans 中的属性会覆盖它
            buffer.set_rich_text(
                font_system,
                spans,
                &default_attrs,
                Shaping::Advanced,
                config.to_cosmic_align(),
            );
            buffer.shape_until_scroll(font_system, false);
            return;
        }
    }
    buffer.set_text(
        font_system,
        text,
        &config.to_cosmic_attrs(),
        Shaping::Advanced,
        config.to_cosmic_align(),
    );
    buffer.shape_until_scroll(font_system, false);
}

/// (meta, TextLayoutConfig)
pub fn prepare_text_buffer(
    font_system: &mut FontSystem,
    text: &str,
    config: &TextLayoutConfig,
    width: f32,
    height: f32,
    dpi_x: f32,
    dpi_y: f32,
    chunk: Option<&[(Attrs, usize, usize)]>,
) -> Buffer {
    let font_size = config.font_size * dpi_y;
    let phys_width = width * dpi_x;
    let phys_height = height * dpi_y;

    let mut buffer = Buffer::new(
        font_system,
        Metrics::new(font_size, font_size * config.line_height_factor),
    );
    buffer.set_size(
        font_system,
        if width > 0.0 { Some(phys_width) } else { None },
        if height > 0.0 {
            Some(phys_height)
        } else {
            None
        },
    );
    // Justified alignment requires word wrapping — cosmic_text only
    // expands inter-word spaces on non-last lines, so without wrapping
    // the single line is treated as "last" and never justified.
    let wrap = if config.text_alignment == TextAlignment::Justified
        && config.to_cosmic_wrap() == Wrap::None
    {
        Wrap::Word
    } else {
        config.to_cosmic_wrap()
    };
    buffer.set_wrap(font_system, wrap);

    let trimming = config.text_trimming;
    let mut final_text = text.to_string();

    if width > 0.0 && trimming != TextTrimming::None {
        // 根据是否有 chunk 选择不同的设置文本方式
        set_text_to_buffer(&mut buffer, font_system, text, chunk, config);

        let do_trim = check_layout_overflow(&buffer, phys_width, phys_height, height);

        if do_trim {
            let is_word = trimming == TextTrimming::EllipsisWord;
            let has_ellipsis = trimming == TextTrimming::EllipsisChar || is_word;
            let chars: Vec<(usize, char)> = text.char_indices().collect();

            let mut l = 0;
            let mut r = chars.len();
            let mut best_text = String::new();

            while l <= r && r < usize::MAX {
                let m = l + (r - l) / 2;
                if m > chars.len() {
                    break;
                }
                let mut test_text = if m < chars.len() {
                    text[..chars[m].0].to_string()
                } else {
                    text.to_string()
                };

                if is_word && m < chars.len() {
                    if let Some(idx) = test_text.rfind(char::is_whitespace) {
                        test_text.truncate(idx);
                    }
                }

                if has_ellipsis {
                    test_text.push_str("...");
                }

                // 根据是否有 chunk 选择不同的设置文本方式
                set_text_to_buffer(&mut buffer, font_system, &test_text, chunk, config);

                let overflow = check_layout_overflow(&buffer, phys_width, phys_height, height);

                if overflow {
                    if m == 0 {
                        break;
                    }
                    r = m - 1;
                } else {
                    best_text = test_text;
                    if l == usize::MAX || m == usize::MAX {
                        break;
                    }
                    l = m + 1;
                }
            }
            final_text = best_text;
        }
    }
    // 设置最终文本
    set_text_to_buffer(&mut buffer, font_system, &final_text, chunk, config);
    buffer
}

pub fn measure_text(
    font_system: &mut FontSystem,
    text: &str,
    config: &TextLayoutConfig,
    width: f32,
    height: f32,
    dpi_x: f32,
    dpi_y: f32,
    chunk: Option<&[(Attrs, usize, usize)]>,
) -> (f32, f32) {
    let buffer = prepare_text_buffer(
        font_system,
        text,
        config,
        width,
        height,
        dpi_x,
        dpi_y,
        chunk,
    );

    let mut max_w = 0.0f32;
    let mut total_h = 0.0f32;
    for run in buffer.layout_runs() {
        max_w = max_w.max(run.line_w);
        total_h += run.line_height;
    }

    (max_w / dpi_x, total_h / dpi_y)
}

pub fn hit_test_point(
    font_system: &mut FontSystem,
    text: &str,
    config: &TextLayoutConfig,
    width: f32,
    height: f32,
    dpi_x: f32,
    dpi_y: f32,
    x: f32,
    y: f32,
    chunk: Option<&[(Attrs, usize, usize)]>,
) -> HitTestResult {
    let phys_height = height * dpi_y;
    let buffer = prepare_text_buffer(
        font_system,
        text,
        config,
        width,
        height,
        dpi_x,
        dpi_y,
        chunk,
    );

    let offset_y = config.calc_offset_y(&buffer, phys_height);

    if let Some(cursor) = buffer.hit(x * dpi_x, y * dpi_y - offset_y) {
        HitTestResult {
            is_inside: true,
            text_index: cursor.index,
            is_trailing: false,
            is_trimmed: false,
            rect: (0.0, 0.0, 0.0, 0.0),
        }
    } else {
        HitTestResult {
            is_inside: false,
            text_index: text.len(),
            is_trailing: true,
            is_trimmed: false,
            rect: (0.0, 0.0, 0.0, 0.0),
        }
    }
}

pub fn hit_test_text_position(
    font_system: &mut FontSystem,
    text: &str,
    config: &TextLayoutConfig,
    width: f32,
    height: f32,
    dpi_x: f32,
    dpi_y: f32,
    text_index: usize,
    trailing: bool,
    chunk: Option<&[(Attrs, usize, usize)]>,
) -> (f32, f32) {
    let phys_height = height * dpi_y;
    let buffer = prepare_text_buffer(
        font_system,
        text,
        config,
        width,
        height,
        dpi_x,
        dpi_y,
        chunk,
    );

    let offset_y = config.calc_offset_y(&buffer, phys_height);

    for run in buffer.layout_runs() {
        for glyph in run.glyphs.iter() {
            if glyph.start <= text_index && glyph.end > text_index {
                let mut gx = glyph.x;
                if trailing {
                    gx += glyph.w;
                }
                return (
                    gx / dpi_x,
                    (run.line_y - run.line_height + glyph.y + offset_y) / dpi_y,
                );
            }
        }
    }

    (0.0, offset_y / dpi_y)
}
