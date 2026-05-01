use crate::graphics::{
    BrushHandle, CosmicTextFormatHandle, HitTestResult, LayoutText, TextAlignment, TextChunk,
    TextLayoutConfig, TextTrimming, FONT_SYSTEM,
};
use crate::types::Rect;
use cosmic_text::{Affinity, Attrs, Buffer, LineIter, Metrics, Shaping, Wrap};
use parking_lot::RwLock;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct CosmicTextLayout<B: BrushHandle, E = ()> {
    pub text: String,
    pub chunks: Option<Vec<TextChunk<B, CosmicTextFormatHandle>>>,
    pub bounds: Rect<f32>,
    pub dpi_scale: (f32, f32),
    dirty: AtomicBool,
    pub buffer: RwLock<Buffer>,
    pub default_text_format: CosmicTextFormatHandle,
    _marker: PhantomData<E>,
}

impl<B: BrushHandle, E> CosmicTextLayout<B, E> {
    pub fn create_text_layout(
        text: String,
        bounds: Rect<f32>,
        default_text_format: CosmicTextFormatHandle,
    ) -> Self {
        Self {
            text,
            chunks: None,
            bounds,
            dpi_scale: (0.0, 0.0),
            dirty: AtomicBool::new(false),
            buffer: RwLock::new(Buffer::new(
                &mut FONT_SYSTEM.lock(),
                Metrics::new(16., 16. * 1.),
            )),
            default_text_format,
            _marker: Default::default(),
        }
    }

    fn ensure_buffer(&self) {
        if self.dirty.load(Ordering::Relaxed) {
            let layout_config = self.prepare_text_chunks();

            let (dpi_x, dpi_y) = self.dpi_scale;
            let (width, height) = (self.bounds.width(), self.bounds.height());

            let buffer = self.prepare_text_buffer(
                &self.text,
                &self.default_text_format.config,
                width,
                height,
                dpi_x,
                dpi_y,
                &layout_config,
            );

            *self.buffer.write() = buffer;
            self.dirty.store(false, Ordering::Relaxed);
        }
    }

    pub fn build(&self) -> parking_lot::RwLockReadGuard<'_, Buffer> {
        self.ensure_buffer();
        self.buffer.read()
    }

    pub fn brush_at(&self, index: usize) -> Option<&B> {
        self.chunks.as_ref()?.get(index).map(|c| &c.brush)
    }

    pub fn chunks(&self) -> Option<&[TextChunk<B, CosmicTextFormatHandle>]> {
        self.chunks.as_ref().map(|v| v.as_slice())
    }

    pub fn buffer(&self) -> parking_lot::RwLockReadGuard<'_, Buffer> {
        self.ensure_buffer();
        self.buffer.read()
    }

    pub fn text_format(&self) -> &CosmicTextFormatHandle {
        &self.default_text_format
    }

    fn prepare_text_chunks(&self) -> Vec<(Attrs<'_>, usize, usize)> {
        let chunks = match &self.chunks {
            Some(c) if !c.is_empty() => c,
            _ => return Vec::new(),
        };

        let len = chunks.len();
        let mut layout_config = Vec::with_capacity(len);

        for (idx, chunk) in chunks.iter().enumerate() {
            let attrs = chunk.text_format.config.to_cosmic_attrs().metadata(idx);
            layout_config.push((attrs, chunk.start, chunk.length));
        }

        layout_config
    }

    fn check_layout_overflow(
        buffer: &Buffer,
        phys_width: f32,
        phys_height: f32,
        height: f32,
    ) -> bool {
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

            if last_end < text_len {
                spans.push((&text[last_end..text_len], default_attrs.clone()));
            }

            buffer.set_rich_text(
                spans,
                &default_attrs,
                Shaping::Advanced,
                config.to_cosmic_align(),
            );
            buffer.shape_until_scroll(&mut FONT_SYSTEM.lock(), false);
            return;
        }
        buffer.set_text(
            text,
            &config.to_cosmic_attrs(),
            Shaping::Advanced,
            config.to_cosmic_align(),
        );
        buffer.shape_until_scroll(&mut FONT_SYSTEM.lock(), false);
    }

    fn prepare_text_buffer(
        &self,
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
            &mut FONT_SYSTEM.lock(),
            Metrics::new(font_size, font_size * config.line_height_factor),
        );
        buffer.set_size(
            if width > 0.0 { Some(phys_width) } else { None },
            if height > 0.0 {
                Some(phys_height)
            } else {
                None
            },
        );
        let wrap = if config.text_alignment == TextAlignment::Justified
            && config.to_cosmic_wrap() == Wrap::None
        {
            Wrap::Word
        } else {
            config.to_cosmic_wrap()
        };
        buffer.set_wrap(wrap);

        let trimming = config.text_trimming;
        let mut final_text = text.to_string();

        if width > 0.0 && trimming != TextTrimming::None {
            Self::set_text_to_buffer(&mut buffer, text, chunks, config);
            let do_trim = Self::check_layout_overflow(&buffer, phys_width, phys_height, height);

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

                    Self::set_text_to_buffer(&mut buffer, &test_text, chunks, config);
                    let overflow =
                        Self::check_layout_overflow(&buffer, phys_width, phys_height, height);

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
        Self::set_text_to_buffer(&mut buffer, &final_text, chunks, config);
        buffer
    }
}

impl<B: BrushHandle, E> LayoutText for CosmicTextLayout<B, E> {
    type Error = E;
    type BrushHandle = B;
    type TextFormatHandle = CosmicTextFormatHandle;

    fn set_font_size(&mut self, font_size: f32) {
        let mut buffer = self.buffer.write();
        buffer.set_metrics(Metrics::new(
            font_size,
            font_size * self.default_text_format.config.line_height_factor,
        ));
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn set_default_text_format(&mut self, default_text_format: Self::TextFormatHandle) {
        self.default_text_format = default_text_format;
        let mut buffer = self.buffer.write();
        let mut metrics = buffer.metrics();
        metrics.line_height =
            metrics.font_size * self.default_text_format.config.line_height_factor;
        buffer.set_metrics(metrics);
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn set_left(&mut self, left: f32) {
        self.bounds.x = left;
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn set_top(&mut self, top: f32) {
        self.bounds.y = top;
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn set_width(&mut self, width: f32) {
        self.bounds.w = width;
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn set_height(&mut self, height: f32) {
        self.bounds.h = height;
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn set_chunks(
        &mut self,
        chunks: Option<Vec<TextChunk<Self::BrushHandle, Self::TextFormatHandle>>>,
    ) {
        self.chunks = chunks;
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn bounds(&self) -> Rect<f32> {
        self.bounds
    }

    fn measure_text(&self) -> Result<(f32, f32), Self::Error> {
        self.ensure_buffer();
        let buffer = self.buffer.read();

        let mut max_w = 0.0f32;
        let mut total_h = 0.0f32;
        for run in buffer.layout_runs() {
            max_w = max_w.max(run.line_w);
            total_h += run.line_height;
        }

        let (dpi_x, dpi_y) = self.dpi_scale;
        Ok((max_w / dpi_x, total_h / dpi_y))
    }

    fn hit_test_point(&self, x: f32, y: f32) -> Result<Option<HitTestResult>, Self::Error> {
        self.ensure_buffer();
        let buffer = self.buffer.read();

        let (dpi_x, dpi_y) = self.dpi_scale;
        let height = self.bounds.height();
        let phys_height = height * dpi_y;
        let phys_x = x * dpi_x;
        let phys_y = y * dpi_y;

        let offset_y = self
            .default_text_format
            .config
            .calc_offset_y(&buffer, phys_height);
        let target_phys_y = phys_y - offset_y;

        let Some(cursor) = buffer.hit(phys_x, target_phys_y) else {
            return Ok(None);
        };

        let global_index = LineIter::new(&self.text)
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

        Ok(Some(HitTestResult {
            global_index,
            line: cursor.line,
            line_offset: cursor.index,
            is_trailing: cursor.affinity == Affinity::After,
            bounds,
            is_inside,
        }))
    }

    fn hit_test_text_position(
        &self,
        text_index: usize,
        trailing: bool,
    ) -> Result<(f32, f32), Self::Error> {
        self.ensure_buffer();
        let buffer = self.buffer.read();

        let (dpi_x, dpi_y) = self.dpi_scale;
        let height = self.bounds.height();
        let phys_height = height * dpi_y;

        let offset_y = self
            .default_text_format
            .config
            .calc_offset_y(&buffer, phys_height);

        let mut line_number = 0;
        let mut line_start = 0;

        for (line_index, (range, _)) in LineIter::new(&self.text).enumerate() {
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
                return Ok((0.0, (run.line_top + offset_y) / dpi_y));
            }

            let last_glyph = run.glyphs.last().unwrap();
            if line_offset >= last_glyph.end {
                return Ok((
                    last_glyph.x / dpi_x + last_glyph.w / dpi_x,
                    (run.line_top + offset_y) / dpi_y,
                ));
            }

            for glyph in run.glyphs.iter() {
                if line_offset == glyph.start {
                    return Ok((glyph.x / dpi_x, (run.line_top + offset_y) / dpi_y));
                } else if line_offset == glyph.end {
                    return Ok((
                        (glyph.x + glyph.w) / dpi_x,
                        (run.line_top + offset_y) / dpi_y,
                    ));
                } else if line_offset > glyph.start && line_offset < glyph.end {
                    let gx = if trailing { glyph.x + glyph.w } else { glyph.x };
                    return Ok((gx / dpi_x, (run.line_top + offset_y) / dpi_y));
                }
            }
        }

        Ok((0.0, offset_y / dpi_y))
    }
}
