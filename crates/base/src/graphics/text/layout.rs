use crate::graphics::{
    BrushHandle, CosmicTextFormatHandle, FontStretch, FontStyle, FontWeight, HitTestResult,
    ParagraphAlignment, TextAlignment, TextChunk, TextTrimming, WordWrapping, FONT_SYSTEM,
};
use crate::types::Rect;
use cosmic_text::{
    Affinity, Align, Attrs, Buffer, Family, LineIter, Metrics, Shaping, Stretch, Style, Weight,
    Wrap,
};
use std::ops::DerefMut;

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
    text: &str,
    chunks: &[(Attrs, usize, usize)],
    config: &TextLayoutConfig,
) {
    if !chunks.is_empty() {
        let mut spans = Vec::new();
        let text_len = text.len();
        let mut last_end = 0;

        // 第一个 chunk 的 metadata 是 0，第二个是 1，依此类推
        // 默认 span 的 metadata = chunks.len()
        let default_metadata = chunks.len();
        let default_attrs = config.to_cosmic_attrs().metadata(default_metadata);

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
            FONT_SYSTEM.lock().deref_mut(),
            spans,
            &default_attrs,
            Shaping::Advanced,
            config.to_cosmic_align(),
        );
        buffer.shape_until_scroll(FONT_SYSTEM.lock().deref_mut(), false);
        return;
    }
    buffer.set_text(
        FONT_SYSTEM.lock().deref_mut(),
        text,
        &config.to_cosmic_attrs(),
        Shaping::Advanced,
        config.to_cosmic_align(),
    );
    buffer.shape_until_scroll(FONT_SYSTEM.lock().deref_mut(), false);
}

/// (meta, TextLayoutConfig)
pub fn prepare_text_buffer(
    text: &str,
    config: &TextLayoutConfig,
    width: f32,
    height: f32,
    dpi_x: f32,
    dpi_y: f32,
    chunks: &[(Attrs, usize, usize)],
) -> Buffer {
    let font_size = config.font_size * dpi_y;
    let phys_width = width * dpi_x;
    let phys_height = height * dpi_y;

    let mut buffer = Buffer::new(
        FONT_SYSTEM.lock().deref_mut(),
        Metrics::new(font_size, font_size * config.line_height_factor),
    );
    buffer.set_size(
        FONT_SYSTEM.lock().deref_mut(),
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
    buffer.set_wrap(FONT_SYSTEM.lock().deref_mut(), wrap);

    let trimming = config.text_trimming;
    let mut final_text = text.to_string();

    if width > 0.0 && trimming != TextTrimming::None {
        // 根据是否有 chunk 选择不同的设置文本方式
        set_text_to_buffer(&mut buffer, text, chunks, config);

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
                set_text_to_buffer(&mut buffer, &test_text, chunks, config);

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
    set_text_to_buffer(&mut buffer, &final_text, chunks, config);
    buffer
}

pub fn measure_text(
    text: &str,
    config: &TextLayoutConfig,
    width: f32,
    height: f32,
    dpi_x: f32,
    dpi_y: f32,
    chunks: &[(Attrs, usize, usize)],
) -> (f32, f32) {
    let buffer = prepare_text_buffer(text, config, width, height, dpi_x, dpi_y, chunks);

    let mut max_w = 0.0f32;
    let mut total_h = 0.0f32;
    for run in buffer.layout_runs() {
        max_w = max_w.max(run.line_w);
        total_h += run.line_height;
    }

    (max_w / dpi_x, total_h / dpi_y)
}

pub fn hit_test_point(
    text: &str,
    config: &TextLayoutConfig,
    width: f32,
    height: f32,
    dpi_x: f32,
    dpi_y: f32,
    x: f32,
    y: f32,
    chunks: &[(Attrs, usize, usize)],
) -> Option<HitTestResult> {
    let phys_height = height * dpi_y;
    let phys_x = x * dpi_x;
    let phys_y = y * dpi_y;
    let buffer = prepare_text_buffer(text, config, width, height, dpi_x, dpi_y, chunks);

    let offset_y = config.calc_offset_y(&buffer, phys_height);
    let target_phys_y = phys_y - offset_y;

    let Some(cursor) = buffer.hit(phys_x, target_phys_y) else {
        return None;
    };

    let global_index = LineIter::new(text)
        .nth(cursor.line)
        .map(|(range, _)| range.start + cursor.index)
        .unwrap_or(cursor.index);

    let mut bounds = None;
    let mut is_inside = false;

    for run in buffer.layout_runs() {
        if run.line_i != cursor.line {
            continue;
        }

        // 垂直判定：y 是否落在这一物理行内
        let line_bottom = run.line_top + run.line_height;
        if target_phys_y >= run.line_top && target_phys_y <= line_bottom {
            // 查找具体的 Glyph
            for glyph in run.glyphs {
                if cursor.index >= glyph.start && cursor.index < glyph.end {
                    let rect = Rect {
                        x: glyph.x / dpi_x,
                        y: (run.line_top + offset_y) / dpi_y, // 加上偏移并转回逻辑
                        w: glyph.w / dpi_x,
                        h: run.line_height / dpi_y,
                    };

                    // 水平判定：使用物理单位进行精确判定
                    if phys_x >= glyph.x && phys_x <= (glyph.x + glyph.w) {
                        is_inside = true;
                    }

                    bounds = Some(rect);
                    break;
                }
            }
        }
    }

    Some(HitTestResult {
        global_index,
        line: cursor.line,
        line_offset: cursor.index,
        is_trailing: cursor.affinity == Affinity::After,
        bounds,
        is_inside,
    })
}

pub fn hit_test_text_position(
    text: &str,
    config: &TextLayoutConfig,
    width: f32,
    height: f32,
    dpi_x: f32,
    dpi_y: f32,
    text_index: usize,
    trailing: bool,
    chunks: &[(Attrs, usize, usize)],
) -> (f32, f32) {
    let phys_height = height * dpi_y;
    let buffer = prepare_text_buffer(text, config, width, height, dpi_x, dpi_y, chunks);

    let offset_y = config.calc_offset_y(&buffer, phys_height);

    let mut line_number = 0;
    let mut line_start = 0;

    for (line_index, (range, _)) in LineIter::new(text).enumerate() {
        if text_index >= range.start && text_index <= range.end {
            line_number = line_index;
            line_start = range.start;
            break;
        }
    }

    let line_offset = text_index - line_start;

    for run in buffer.layout_runs() {
        if run.line_i != line_number {
            continue;
        }

        if run.glyphs.is_empty() {
            return (0.0, (run.line_top + offset_y) / dpi_y);
        }

        let last_glyph = run.glyphs.last().unwrap();
        if line_offset >= last_glyph.end {
            return (
                last_glyph.x / dpi_x + last_glyph.w / dpi_x,
                (run.line_top + offset_y) / dpi_y,
            );
        }

        for glyph in run.glyphs.iter() {
            if line_offset == glyph.start {
                return (glyph.x / dpi_x, (run.line_top + offset_y) / dpi_y);
            } else if line_offset == glyph.end {
                let x = (glyph.x + glyph.w) / dpi_x;
                dbg!(x);
                return (
                    (glyph.x + glyph.w) / dpi_x,
                    (run.line_top + offset_y) / dpi_y,
                );
            } else if line_offset > glyph.start && line_offset < glyph.end {
                let gx = if trailing { glyph.x + glyph.w } else { glyph.x };
                return (gx / dpi_x, (run.line_top + offset_y) / dpi_y);
            }
        }
    }

    (0.0, offset_y / dpi_y)
}

#[inline]
pub fn prepare_text_chunks<'a, B: BrushHandle>(
    chunks: Option<&'a [TextChunk<'a, B, CosmicTextFormatHandle>]>,
    brushes: Option<&mut Vec<&'a B>>,
) -> Vec<(Attrs<'a>, usize, usize)> {
    let chunks = match chunks {
        Some(c) if !c.is_empty() => c,
        _ => return Vec::new(),
    };

    let len = chunks.len();
    let mut layout_config = Vec::with_capacity(len);

    if let Some(b_vec) = brushes {
        for chunk in chunks {
            b_vec.push(chunk.brush);
            let attrs = chunk
                .text_format
                .config
                .to_cosmic_attrs()
                .metadata(b_vec.len() - 1);

            layout_config.push((attrs, chunk.start, chunk.length));
        }
    } else {
        for (idx, chunk) in chunks.iter().enumerate() {
            let attrs = chunk.text_format.config.to_cosmic_attrs().metadata(idx);

            layout_config.push((attrs, chunk.start, chunk.length));
        }
    }

    layout_config
}
