use flor_graphics_base::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextFormatHandle,
    TextTrimming, WordWrapping,
};
use windows::Win32::Graphics::DirectWrite::*;
#[derive(Debug, Clone)]
pub struct D2DTextFormatHandle {
    raw: IDWriteTextFormat,

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
    dirty: bool,
}

impl D2DTextFormatHandle {
    pub fn new(raw: IDWriteTextFormat, family_name: impl Into<String>) -> Self {
        Self {
            raw,
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
            dirty: true, // 初始状态为脏，确保第一次会创建
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

    pub fn raw(&self) -> &IDWriteTextFormat {
        &self.raw
    }

    pub fn set_raw(&mut self, raw: IDWriteTextFormat) {
        self.raw = raw;
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
            self.dirty = true;
        }
        self
    }
    fn font_size(&self) -> f32 {
        self.size
    }

    fn set_font_weight(&mut self, weight: FontWeight) -> &mut Self {
        if self.weight != weight {
            self.weight = weight;
            self.dirty = true;
        }
        self
    }
    fn font_weight(&self) -> FontWeight {
        self.weight
    }

    fn set_font_style(&mut self, style: FontStyle) -> &mut Self {
        if self.style != style {
            self.style = style;
            self.dirty = true;
        }
        self
    }
    fn font_style(&self) -> FontStyle {
        self.style
    }

    fn set_font_stretch(&mut self, stretch: FontStretch) -> &mut Self {
        if self.stretch != stretch {
            self.stretch = stretch;
            self.dirty = true;
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
            self.dirty = true;
        }
        self
    }
    fn text_alignment(&self) -> TextAlignment {
        self.text_align
    }

    fn set_paragraph_alignment(&mut self, align: ParagraphAlignment) -> &mut Self {
        if self.para_align != align {
            self.para_align = align;
            self.dirty = true;
        }
        self
    }
    fn paragraph_alignment(&self) -> ParagraphAlignment {
        self.para_align
    }

    fn set_word_wrapping(&mut self, wrapping: WordWrapping) -> &mut Self {
        if self.wrapping != wrapping {
            self.wrapping = wrapping;
            self.dirty = true;
        }
        self
    }
    fn word_wrapping(&self) -> WordWrapping {
        self.wrapping
    }

    fn set_line_height(&mut self, line_height_factor: f32) -> &mut Self {
        if (self.line_height - line_height_factor).abs() > f32::EPSILON {
            self.line_height = line_height_factor;
            self.dirty = true;
        }
        self
    }
    fn line_height(&self) -> f32 {
        self.line_height
    }

    fn set_text_trimming(&mut self, trimming: TextTrimming) -> &mut Self {
        if self.trimming != trimming {
            self.trimming = trimming;
            self.dirty = true;
        }
        self
    }
    fn text_trimming(&self) -> TextTrimming {
        self.trimming
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
