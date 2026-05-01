use crate::encode::encode_unicode;
use crate::error::D2DError;
use crate::handle::{D2DBrushHandle, D2DTextFormatHandle};
use crate::render_factory::RenderFactory;
use flor_base::graphics::{HitTestResult, LayoutText, TextChunk, TextFormatHandle};
use flor_base::types::Rect;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::core::{BOOL, HSTRING};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteTextLayout, DWRITE_HIT_TEST_METRICS, DWRITE_TEXT_METRICS, DWRITE_TEXT_RANGE,
};

#[derive(Debug)]
pub struct D2DTextLayout {
    text: String,
    bounds: Rect<f32>,
    default_text_format: D2DTextFormatHandle,
    chunks: Option<Vec<TextChunk<D2DBrushHandle, D2DTextFormatHandle>>>,
    /// 缓存的 DirectWrite TextLayout
    d2d_text_layout: Arc<Mutex<Option<IDWriteTextLayout>>>,
    /// DPI 缩放
    dpi_scale: (f32, f32),
    /// 是否需要重新创建布局
    dirty: Arc<Mutex<bool>>,
}

impl D2DTextLayout {
    pub fn create_text_layout(
        text: String,
        bounds: Rect<f32>,
        default_text_format: D2DTextFormatHandle,
    ) -> Self {
        Self {
            text,
            bounds,
            default_text_format,
            chunks: None,
            d2d_text_layout: Arc::new(Mutex::new(None)),
            dpi_scale: (1.0, 1.0),
            dirty: Arc::new(Mutex::new(true)),
        }
    }

    fn ensure_layout(&self) -> Result<(), D2DError> {
        let mut dirty_guard = self.dirty.lock();
        if !*dirty_guard {
            return Ok(());
        }

        // 重建布局
        let text_utf16 = encode_unicode(&self.text);

        // 确保 text_format 是最新的
        self.default_text_format.rebuild()?;

        let dwrite_layout = unsafe {
            RenderFactory::get().write_factory.CreateTextLayout(
                &text_utf16,
                &self.default_text_format.raw(),
                self.bounds.w,
                self.bounds.h,
            )?
        };

        // 应用 chunks
        if let Some(chunks) = &self.chunks {
            Self::apply_chunks(&dwrite_layout, &self.text, chunks)?;
        }

        *self.d2d_text_layout.lock() = Some(dwrite_layout);
        *dirty_guard = false;
        Ok(())
    }

    /// 应用文本块
    fn apply_chunks(
        text_layout: &IDWriteTextLayout,
        text: &str,
        chunks: &[TextChunk<D2DBrushHandle, D2DTextFormatHandle>],
    ) -> Result<(), D2DError> {
        // 辅助函数：将 UTF-8 索引转换为 UTF-16 索引
        let utf8_to_utf16 = |text: &str, utf8_idx: usize| -> usize {
            let mut utf16_idx = 0;
            let mut current_utf8 = 0;
            for c in text.chars() {
                if current_utf8 >= utf8_idx {
                    break;
                }
                utf16_idx += c.len_utf16();
                current_utf8 += c.len_utf8();
            }
            utf16_idx
        };

        unsafe {
            for chunk in chunks {
                let end = chunk.start + chunk.length;
                if end <= text.len() {
                    // 确保这个 chunk 的 text_format 也是最新的
                    chunk.text_format.rebuild()?;

                    let start_utf16 = utf8_to_utf16(text, chunk.start);
                    let end_utf16 = utf8_to_utf16(text, end);
                    let chunk_range = DWRITE_TEXT_RANGE {
                        startPosition: start_utf16 as u32,
                        length: (end_utf16 - start_utf16) as u32,
                    };

                    // 设置画笔
                    text_layout.SetDrawingEffect(chunk.brush.raw(), chunk_range)?;

                    // 设置格式
                    let (weight, style, stretch) = chunk.text_format.map_props();

                    let family_name = HSTRING::from(chunk.text_format.font_family_name());
                    text_layout.SetFontFamilyName(&family_name, chunk_range)?;
                    text_layout.SetFontSize(chunk.text_format.font_size(), chunk_range)?;
                    text_layout.SetFontWeight(weight, chunk_range)?;
                    text_layout.SetFontStyle(style, chunk_range)?;
                    text_layout.SetFontStretch(stretch, chunk_range)?;
                }
            }
        }
        Ok(())
    }

    /// 标记为需要重新创建
    fn mark_dirty(&self) {
        *self.dirty.lock() = true;
    }

    /// 获取底层的 DirectWrite TextLayout
    pub fn dwrite_layout(&self) -> Result<IDWriteTextLayout, D2DError> {
        self.ensure_layout()?;
        let guard = self.d2d_text_layout.lock();
        guard.as_ref().cloned().ok_or_else(|| {
            windows::core::Error::from(windows::Win32::Foundation::E_UNEXPECTED).into()
        })
    }

    /// 设置 DPI 缩放
    pub fn set_dpi_scale(&mut self, dpi_x: f32, dpi_y: f32) {
        self.dpi_scale = (dpi_x, dpi_y);
    }

    /// 获取文本内容
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 获取默认文本格式
    pub fn default_text_format(&self) -> &D2DTextFormatHandle {
        &self.default_text_format
    }

    /// 获取文本块
    pub fn chunks(&self) -> Option<&[TextChunk<D2DBrushHandle, D2DTextFormatHandle>]> {
        self.chunks.as_deref()
    }
}

impl LayoutText for D2DTextLayout {
    type Error = D2DError;
    type BrushHandle = D2DBrushHandle;
    type TextFormatHandle = D2DTextFormatHandle;

    fn set_font_size(&mut self, size: f32) {
        self.default_text_format.set_font_size(size);
        self.mark_dirty();
    }

    fn set_default_text_format(&mut self, text_format: Self::TextFormatHandle) {
        self.default_text_format = text_format;
        self.mark_dirty();
    }

    fn set_left(&mut self, left: f32) {
        self.bounds.x = left;
        self.mark_dirty();
    }

    fn set_top(&mut self, top: f32) {
        self.bounds.y = top;
        self.mark_dirty();
    }

    fn set_width(&mut self, width: f32) {
        self.bounds.w = width;
        self.mark_dirty();
    }

    fn set_height(&mut self, height: f32) {
        self.bounds.h = height;
        self.mark_dirty();
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
        self.mark_dirty();
    }

    fn set_chunks(
        &mut self,
        chunks: Option<Vec<TextChunk<Self::BrushHandle, Self::TextFormatHandle>>>,
    ) {
        self.chunks = chunks;
        self.mark_dirty();
    }

    fn bounds(&self) -> Rect<f32> {
        self.bounds
    }

    fn measure_text(&self) -> Result<(f32, f32), Self::Error> {
        self.ensure_layout()?;
        let layout = self.dwrite_layout()?;

        unsafe {
            let mut metrics = DWRITE_TEXT_METRICS::default();
            layout.GetMetrics(&mut metrics)?;
            Ok((metrics.width, metrics.height))
        }
    }

    fn hit_test_point(&self, x: f32, y: f32) -> Result<Option<HitTestResult>, Self::Error> {
        self.ensure_layout()?;
        let layout = self.dwrite_layout()?;

        unsafe {
            let mut is_trailing = BOOL(0);
            let mut is_inside = BOOL(0);
            let mut metrics = DWRITE_HIT_TEST_METRICS::default();

            layout.HitTestPoint(x, y, &mut is_trailing, &mut is_inside, &mut metrics)?;

            // 将 UTF-16 位置转换回 UTF-8
            let mut utf8_index = 0;
            let mut utf16_index = 0;

            while utf16_index < metrics.textPosition as usize && utf8_index < self.text.len() {
                let c = self.text[utf8_index..].chars().next().unwrap();
                utf16_index += c.len_utf16();
                utf8_index += c.len_utf8();
            }

            Ok(Some(HitTestResult {
                global_index: utf8_index,
                line: 0,                 // 暂时简化
                line_offset: utf8_index, // 暂时简化
                is_trailing: is_trailing.as_bool(),
                bounds: Some(Rect {
                    x: metrics.left,
                    y: metrics.top,
                    w: metrics.width,
                    h: metrics.height,
                }),
                is_inside: is_inside.as_bool(),
            }))
        }
    }

    fn hit_test_text_position(
        &self,
        text_index: usize,
        trailing: bool,
    ) -> Result<(f32, f32), Self::Error> {
        self.ensure_layout()?;
        let layout = self.dwrite_layout()?;

        // 将 UTF-8 索引转换为 UTF-16
        let mut utf16_index = 0;
        let mut utf8_index = 0;

        while utf8_index < text_index && utf8_index < self.text.len() {
            let c = self.text[utf8_index..].chars().next().unwrap();
            utf16_index += c.len_utf16();
            utf8_index += c.len_utf8();
        }

        unsafe {
            let mut x = 0.0f32;
            let mut y = 0.0f32;
            let mut metrics = DWRITE_HIT_TEST_METRICS::default();

            layout.HitTestTextPosition(
                utf16_index as u32,
                trailing,
                &mut x,
                &mut y,
                &mut metrics,
            )?;

            Ok((x, y))
        }
    }
}
