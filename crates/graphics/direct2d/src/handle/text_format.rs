use crate::error::D2DError;
use crate::render_factory::RenderFactory;
use flor_base::graphics::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextFormatHandle,
    TextTrimming, WordWrapping,
};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::HSTRING;
use windows::Win32::Graphics::DirectWrite::*;
#[derive(Debug, Clone)]
pub struct D2DTextFormatHandle {
    raw: Arc<Mutex<IDWriteTextFormat>>,

    family_name: String,
    size: f32,
    weight: FontWeight,
    style: FontStyle,
    stretch: FontStretch,

    text_align: TextAlignment,
    para_align: ParagraphAlignment,
    wrapping: WordWrapping,
    line_height: f32,
    trimming: TextTrimming,

    // 脏标记：一旦为 true，下次 rebuild 时需要重建底层对象
    dirty: Arc<AtomicBool>,
}

impl D2DTextFormatHandle {
    pub fn new(raw: IDWriteTextFormat, family_name: impl Into<String>) -> Self {
        Self {
            raw: Arc::new(Mutex::new(raw)),
            family_name: family_name.into(),
            size: 16.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            text_align: TextAlignment::Start,
            para_align: ParagraphAlignment::Top,
            wrapping: WordWrapping::NoWrap,
            line_height: 1.0,
            trimming: TextTrimming::None,
            dirty: Arc::new(AtomicBool::new(true)), // 初始状态为脏，确保第一次会创建
        }
    }

    pub(crate) fn map_props(&self) -> (DWRITE_FONT_WEIGHT, DWRITE_FONT_STYLE, DWRITE_FONT_STRETCH) {
        let w = match self.font_weight() {
            FontWeight::Thin => DWRITE_FONT_WEIGHT_THIN,
            FontWeight::ExtraLight => DWRITE_FONT_WEIGHT_EXTRA_LIGHT,
            FontWeight::Light => DWRITE_FONT_WEIGHT_LIGHT,
            FontWeight::Normal => DWRITE_FONT_WEIGHT_NORMAL,
            FontWeight::Medium => DWRITE_FONT_WEIGHT_MEDIUM,
            FontWeight::SemiBold => DWRITE_FONT_WEIGHT_SEMI_BOLD,
            FontWeight::Bold => DWRITE_FONT_WEIGHT_BOLD,
            FontWeight::ExtraBold => DWRITE_FONT_WEIGHT_EXTRA_BOLD,
            FontWeight::Black => DWRITE_FONT_WEIGHT_BLACK,
            FontWeight::ExtraBlack => DWRITE_FONT_WEIGHT_EXTRA_BLACK,
        };

        let s = match self.font_style() {
            FontStyle::Normal => DWRITE_FONT_STYLE_NORMAL,
            FontStyle::Oblique => DWRITE_FONT_STYLE_OBLIQUE,
            FontStyle::Italic => DWRITE_FONT_STYLE_ITALIC,
        };

        let str = match self.font_stretch() {
            FontStretch::UltraCondensed => DWRITE_FONT_STRETCH_ULTRA_CONDENSED,
            FontStretch::ExtraCondensed => DWRITE_FONT_STRETCH_EXTRA_CONDENSED,
            FontStretch::Condensed => DWRITE_FONT_STRETCH_CONDENSED,
            FontStretch::SemiCondensed => DWRITE_FONT_STRETCH_SEMI_CONDENSED,
            FontStretch::Normal => DWRITE_FONT_STRETCH_NORMAL,
            FontStretch::SemiExpanded => DWRITE_FONT_STRETCH_SEMI_EXPANDED,
            FontStretch::Expanded => DWRITE_FONT_STRETCH_EXPANDED,
            FontStretch::ExtraExpanded => DWRITE_FONT_STRETCH_EXTRA_EXPANDED,
            FontStretch::UltraExpanded => DWRITE_FONT_STRETCH_ULTRA_EXPANDED,
        };
        (w, s, str)
    }

    pub fn raw(&self) -> IDWriteTextFormat {
        self.raw.lock().clone()
    }

    pub fn set_raw(&self, raw: IDWriteTextFormat) {
        *self.raw.lock() = raw;
    }
}

impl TextFormatHandle for D2DTextFormatHandle {
    // =========================================================
    // Getters & Setters
    // 逻辑统一：Set 改 State 并标记 Dirty，Get 读 State
    // =========================================================

    fn set_font_size(&mut self, size: f32) -> &mut Self {
        if (self.size - size).abs() > f32::EPSILON {
            self.size = size;
            self.make_dirty();
        }
        self
    }
    fn font_size(&self) -> f32 {
        self.size
    }

    fn set_font_weight(&mut self, weight: FontWeight) -> &mut Self {
        if self.weight != weight {
            self.weight = weight;
            self.make_dirty();
        }
        self
    }
    fn font_weight(&self) -> FontWeight {
        self.weight
    }

    fn set_font_style(&mut self, style: FontStyle) -> &mut Self {
        if self.style != style {
            self.style = style;
            self.make_dirty();
        }
        self
    }
    fn font_style(&self) -> FontStyle {
        self.style
    }

    fn set_font_stretch(&mut self, stretch: FontStretch) -> &mut Self {
        if self.stretch != stretch {
            self.stretch = stretch;
            self.make_dirty();
        }
        self
    }
    fn font_stretch(&self) -> FontStretch {
        self.stretch
    }

    fn font_family_name(&self) -> String {
        self.family_name.clone()
    }

    fn set_text_alignment(&mut self, align: TextAlignment) -> &mut Self {
        if self.text_align != align {
            self.text_align = align;
            self.make_dirty();
        }
        self
    }
    fn text_alignment(&self) -> TextAlignment {
        self.text_align
    }

    fn set_paragraph_alignment(&mut self, align: ParagraphAlignment) -> &mut Self {
        if self.para_align != align {
            self.para_align = align;
            self.make_dirty();
        }
        self
    }
    fn paragraph_alignment(&self) -> ParagraphAlignment {
        self.para_align
    }

    fn set_word_wrapping(&mut self, wrapping: WordWrapping) -> &mut Self {
        if self.wrapping != wrapping {
            self.wrapping = wrapping;
            self.make_dirty();
        }
        self
    }
    fn word_wrapping(&self) -> WordWrapping {
        self.wrapping
    }

    fn set_line_height(&mut self, line_height_factor: f32) -> &mut Self {
        if (self.line_height - line_height_factor).abs() > f32::EPSILON {
            self.line_height = line_height_factor;
            self.make_dirty();
        }
        self
    }
    fn line_height(&self) -> f32 {
        self.line_height
    }

    fn set_text_trimming(&mut self, trimming: TextTrimming) -> &mut Self {
        if self.trimming != trimming {
            self.trimming = trimming;
            self.make_dirty();
        }
        self
    }
    fn text_trimming(&self) -> TextTrimming {
        self.trimming
    }
}

impl D2DTextFormatHandle {
    #[inline]
    pub fn make_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }
    pub fn dirty(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
    }

    pub fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Release);
    }

    pub fn rebuild(&self) -> Result<(), D2DError> {
        // 1. 极速路径 (Fast Path)
        if !self.dirty() {
            return Ok(());
        }

        // 2. 准备不可变参数
        let family = HSTRING::from(&self.font_family_name());
        let (weight, style, stretch) = self.map_props();
        let locale = HSTRING::from("en-us");

        // 3. 创建核心对象
        let new_text_format = unsafe {
            RenderFactory::get().write_factory.CreateTextFormat(
                windows::core::PCWSTR(family.as_ptr()),
                None,
                weight,
                style,
                stretch,
                self.font_size(),
                windows::core::PCWSTR(locale.as_ptr()),
            )?
        };

        // 4. 配置可变属性
        unsafe {
            // Alignment
            let dw_text_align = match self.text_alignment() {
                TextAlignment::Start => DWRITE_TEXT_ALIGNMENT_LEADING,
                TextAlignment::End => DWRITE_TEXT_ALIGNMENT_TRAILING,
                TextAlignment::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
                TextAlignment::Justified => DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
            };
            new_text_format.SetTextAlignment(dw_text_align)?;

            let dw_para_align = match self.paragraph_alignment() {
                ParagraphAlignment::Top => DWRITE_PARAGRAPH_ALIGNMENT_NEAR,
                ParagraphAlignment::Center => DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
                ParagraphAlignment::Bottom => DWRITE_PARAGRAPH_ALIGNMENT_FAR,
            };
            new_text_format.SetParagraphAlignment(dw_para_align)?;

            // Wrapping
            let dw_wrap = if self.text_alignment() == TextAlignment::Justified
                && self.word_wrapping() == WordWrapping::NoWrap
            {
                DWRITE_WORD_WRAPPING_WRAP
            } else {
                match self.word_wrapping() {
                    WordWrapping::NoWrap => DWRITE_WORD_WRAPPING_NO_WRAP,
                    WordWrapping::Wrap => DWRITE_WORD_WRAPPING_WRAP,
                    WordWrapping::Character => DWRITE_WORD_WRAPPING_CHARACTER,
                }
            };
            new_text_format.SetWordWrapping(dw_wrap)?;

            // Line Height
            if (self.line_height() - 1.0).abs() > f32::EPSILON {
                let line_spacing = self.font_size() * self.line_height();
                let extra_space = line_spacing - self.font_size();
                let baseline = extra_space / 2.0 + self.font_size() * 0.85;

                new_text_format.SetLineSpacing(
                    DWRITE_LINE_SPACING_METHOD_UNIFORM,
                    line_spacing,
                    baseline,
                )?;
            } else {
                new_text_format.SetLineSpacing(DWRITE_LINE_SPACING_METHOD_DEFAULT, 0.0, 0.0)?;
            }

            // Trimming
            let (granularity, _delimiter) = match self.text_trimming() {
                TextTrimming::None => (DWRITE_TRIMMING_GRANULARITY_NONE, 0),
                TextTrimming::Character | TextTrimming::EllipsisChar => {
                    (DWRITE_TRIMMING_GRANULARITY_CHARACTER, 0)
                }
                TextTrimming::Word | TextTrimming::EllipsisWord => {
                    (DWRITE_TRIMMING_GRANULARITY_WORD, 0)
                }
            };
            new_text_format.SetTrimming(
                &DWRITE_TRIMMING {
                    granularity,
                    delimiter: 0,
                    delimiterCount: 0,
                },
                None,
            )?;
        }
        self.set_raw(new_text_format);
        self.clear_dirty();
        Ok(())
    }
}
