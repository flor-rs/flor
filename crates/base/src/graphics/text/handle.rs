use crate::graphics::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextFormatHandle,
    TextLayoutConfig, TextTrimming, WordWrapping,
};
use cosmic_text::fontdb::{Source, ID};
use cosmic_text::FontSystem;
use once_cell::sync::Lazy;
use parking_lot::Mutex;

pub static FONT_SYSTEM: Lazy<Mutex<FontSystem>> = Lazy::new(|| Mutex::new(FontSystem::new()));

#[derive(Debug, Clone)]
pub struct CosmicTextFormatHandle {
    pub config: TextLayoutConfig,
    pub custom_font_id: Option<ID>,
}

impl CosmicTextFormatHandle {
    pub fn new_with_font_family_name(font_family_name: &str) -> Self {
        Self {
            config: TextLayoutConfig {
                font_family_name: font_family_name.to_string(),
                ..Default::default()
            },
            custom_font_id: None,
        }
    }
    pub fn new_with_font_source(font_source: Source) -> Self {
        let mut font_system_lock = FONT_SYSTEM.lock();
        let ids = font_system_lock.db_mut().load_font_source(font_source);
        Self {
            config: TextLayoutConfig {
                ..Default::default()
            },
            custom_font_id: if !ids.is_empty() { Some(ids[0]) } else { None },
        }
    }
}

impl Drop for CosmicTextFormatHandle {
    fn drop(&mut self) {
        if let Some(id) = self.custom_font_id {
            let mut font_system_lock = FONT_SYSTEM.lock();
            font_system_lock.db_mut().remove_face(id);
        }
    }
}

impl TextFormatHandle for CosmicTextFormatHandle {
    fn set_font_size(&mut self, size: f32) -> &mut Self {
        self.config.font_size = size;
        self
    }

    fn font_size(&self) -> f32 {
        self.config.font_size
    }

    fn set_font_weight(&mut self, weight: FontWeight) -> &mut Self {
        self.config.font_weight = weight;
        self
    }

    fn font_weight(&self) -> FontWeight {
        self.config.font_weight
    }

    fn set_font_style(&mut self, style: FontStyle) -> &mut Self {
        self.config.font_style = style;
        self
    }

    fn font_style(&self) -> FontStyle {
        self.config.font_style
    }

    fn set_font_stretch(&mut self, stretch: FontStretch) -> &mut Self {
        self.config.font_stretch = stretch;
        self
    }

    fn font_stretch(&self) -> FontStretch {
        self.config.font_stretch
    }

    fn font_family_name(&self) -> String {
        self.config.font_family_name.clone()
    }

    fn set_text_alignment(&mut self, align: TextAlignment) -> &mut Self {
        self.config.text_alignment = align;
        self
    }

    fn text_alignment(&self) -> TextAlignment {
        self.config.text_alignment
    }

    fn set_paragraph_alignment(&mut self, align: ParagraphAlignment) -> &mut Self {
        self.config.paragraph_alignment = align;
        self
    }

    fn paragraph_alignment(&self) -> ParagraphAlignment {
        self.config.paragraph_alignment
    }

    fn set_word_wrapping(&mut self, wrapping: WordWrapping) -> &mut Self {
        self.config.word_wrapping = wrapping;
        self
    }

    fn word_wrapping(&self) -> WordWrapping {
        self.config.word_wrapping
    }

    fn set_line_height(&mut self, line_height_factor: f32) -> &mut Self {
        self.config.line_height_factor = line_height_factor;
        self
    }

    fn line_height(&self) -> f32 {
        self.config.line_height_factor
    }

    fn set_text_trimming(&mut self, trimming: TextTrimming) -> &mut Self {
        self.config.text_trimming = trimming;
        self
    }

    fn text_trimming(&self) -> TextTrimming {
        self.config.text_trimming
    }
}
