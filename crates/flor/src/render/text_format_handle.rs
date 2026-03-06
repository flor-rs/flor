#[cfg(feature = "tiny-skia")]
use crate::graphics_cpu::handle::TinySkiaTextFormatHandle;
#[cfg(feature = "direct2d")]
use crate::graphics_gpu::handle::D2DTextFormatHandle;
#[cfg(feature = "opengl")]
use crate::graphics_gpu::handle::GlTextFormatHandle;
use flor_base::graphics::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextFormatHandle,
    TextTrimming, WordWrapping,
};

#[derive(Debug, Clone)]
pub enum FlorTextFormatHandle {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DTextFormatHandle,
        #[cfg(feature = "opengl")] GlTextFormatHandle,
    ),
    #[cfg(feature = "cpu-render-backend")]
    CPU(#[cfg(feature = "tiny-skia")] TinySkiaTextFormatHandle),
}

impl TextFormatHandle for FlorTextFormatHandle {
    // =========================================================
    // Setters (分发副作用，最后返回外层的 self)
    // =========================================================

    fn set_font_size(&mut self, size: f32) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_font_size(size);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_font_size(size);
            }
        }
        self
    }

    fn font_size(&self) -> f32 {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.font_size(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.font_size(),
        }
    }

    fn set_font_weight(&mut self, weight: FontWeight) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_font_weight(weight);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_font_weight(weight);
            }
        }
        self
    }

    fn font_weight(&self) -> FontWeight {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.font_weight(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.font_weight(),
        }
    }

    fn set_font_style(&mut self, style: FontStyle) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_font_style(style);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_font_style(style);
            }
        }
        self
    }

    fn font_style(&self) -> FontStyle {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.font_style(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.font_style(),
        }
    }

    fn set_font_stretch(&mut self, stretch: FontStretch) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_font_stretch(stretch);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_font_stretch(stretch);
            }
        }
        self
    }

    fn font_stretch(&self) -> FontStretch {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.font_stretch(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.font_stretch(),
        }
    }

    fn font_family_name(&self) -> String {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.font_family_name(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.font_family_name(),
        }
    }

    // =========================================================
    // Getters (直接返回内部值)
    // =========================================================

    fn set_text_alignment(&mut self, align: TextAlignment) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_text_alignment(align);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_text_alignment(align);
            }
        }
        self
    }

    fn text_alignment(&self) -> TextAlignment {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.text_alignment(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.text_alignment(),
        }
    }

    fn set_paragraph_alignment(&mut self, align: ParagraphAlignment) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_paragraph_alignment(align);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_paragraph_alignment(align);
            }
        }
        self
    }

    fn paragraph_alignment(&self) -> ParagraphAlignment {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.paragraph_alignment(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.paragraph_alignment(),
        }
    }

    fn set_word_wrapping(&mut self, wrapping: WordWrapping) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_word_wrapping(wrapping);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_word_wrapping(wrapping);
            }
        }
        self
    }

    fn word_wrapping(&self) -> WordWrapping {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.word_wrapping(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.word_wrapping(),
        }
    }

    fn set_line_height(&mut self, line_height_factor: f32) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_line_height(line_height_factor);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_line_height(line_height_factor);
            }
        }
        self
    }

    fn line_height(&self) -> f32 {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.line_height(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.line_height(),
        }
    }

    fn set_text_trimming(&mut self, trimming: TextTrimming) -> &mut Self {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                h.set_text_trimming(trimming);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                h.set_text_trimming(trimming);
            }
        }
        self
    }

    fn text_trimming(&self) -> TextTrimming {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.text_trimming(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.text_trimming(),
        }
    }
}
