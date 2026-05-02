use crate::render::FlorTextFormatHandle;
use crate::render::{FlorBrushHandle, FlorRendererError};
use flor_base::graphics::{HitTestResult, LayoutText, TextChunk};
use flor_base::types::Rect;
#[cfg(feature = "tiny-skia")]
use graphics_cpu::text_layout::TinySkiaTextLayout;
#[cfg(feature = "direct2d")]
use graphics_gpu::text_layout::D2DTextLayout;
#[cfg(feature = "opengl")]
use graphics_gpu::text_layout::GlTextLayout;

#[derive(Debug)]
pub enum FlorLayoutText {
    #[cfg(feature = "gpu-render-backend")]
    GPU(
        #[cfg(feature = "direct2d")] D2DTextLayout,
        #[cfg(feature = "opengl")] GlTextLayout,
    ),
    #[cfg(feature = "cpu-render-backend")]
    CPU(#[cfg(feature = "tiny-skia")] TinySkiaTextLayout),
}

impl LayoutText for FlorLayoutText {
    type Error = FlorRendererError;
    type BrushHandle = FlorBrushHandle;
    type TextFormatHandle = FlorTextFormatHandle;

    fn set_font_size(&mut self, size: f32) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.set_font_size(size),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.set_font_size(size),
        }
    }

    fn set_default_text_format(&mut self, text_format: Self::TextFormatHandle) {
        match (self, text_format) {
            #[cfg(feature = "gpu-render-backend")]
            (Self::GPU(h), FlorTextFormatHandle::GPU(fmt)) => {
                h.set_default_text_format(fmt);
            }
            #[cfg(feature = "cpu-render-backend")]
            (Self::CPU(h), FlorTextFormatHandle::CPU(fmt)) => {
                h.set_default_text_format(fmt);
            }
            #[cfg(all(feature = "gpu-render-backend", feature = "cpu-render-backend"))]
            _ => {}
        }
    }

    fn set_left(&mut self, left: f32) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.set_left(left),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.set_left(left),
        }
    }

    fn set_top(&mut self, top: f32) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.set_top(top),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.set_top(top),
        }
    }

    fn set_width(&mut self, width: f32) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.set_width(width),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.set_width(width),
        }
    }

    fn set_height(&mut self, height: f32) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.set_height(height),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.set_height(height),
        }
    }

    fn set_text(&mut self, text: String) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.set_text(text),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.set_text(text),
        }
    }

    fn set_chunks(
        &mut self,
        chunks: Option<Vec<TextChunk<Self::BrushHandle, Self::TextFormatHandle>>>,
    ) {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => {
                let converted = chunks.and_then(|c| {
                    let mut result: Vec<_> = c
                        .into_iter()
                        .filter_map(|chunk| match (chunk.brush, chunk.text_format) {
                            (
                                FlorBrushHandle::GPU(brush),
                                FlorTextFormatHandle::GPU(text_format),
                            ) => Some(TextChunk {
                                brush,
                                text_format,
                                start: chunk.start,
                                length: chunk.length,
                            }),
                            _ => None,
                        })
                        .collect();

                    // 排序
                    result.sort_unstable_by_key(|chunk| chunk.start);

                    // 检查重叠
                    let mut last_end = 0;
                    let mut valid = true;
                    for chunk in &result {
                        if chunk.start < last_end {
                            valid = false;
                            break;
                        }
                        last_end = chunk.start + chunk.length;
                    }

                    if valid {
                        Some(result)
                    } else {
                        None
                    }
                });
                h.set_chunks(converted);
            }
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => {
                let converted = chunks.and_then(|c| {
                    let mut result: Vec<_> = c
                        .into_iter()
                        .filter_map(|chunk| match (chunk.brush, chunk.text_format) {
                            (
                                FlorBrushHandle::CPU(brush),
                                FlorTextFormatHandle::CPU(text_format),
                            ) => Some(TextChunk {
                                brush,
                                text_format,
                                start: chunk.start,
                                length: chunk.length,
                            }),
                            _ => None,
                        })
                        .collect();

                    // 排序
                    result.sort_unstable_by_key(|chunk| chunk.start);

                    // 检查重叠
                    let mut last_end = 0;
                    let mut valid = true;
                    for chunk in &result {
                        if chunk.start < last_end {
                            valid = false;
                            break;
                        }
                        last_end = chunk.start + chunk.length;
                    }

                    if valid {
                        Some(result)
                    } else {
                        None
                    }
                });
                h.set_chunks(converted);
            }
        }
    }

    fn bounds(&self) -> Rect<f32> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => h.bounds(),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => h.bounds(),
        }
    }

    fn measure_text(&self) -> Result<(f32, f32), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => Ok(h.measure_text()?),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => Ok(h.measure_text()?),
        }
    }

    fn hit_test_point(&self, x: f32, y: f32) -> Result<Option<HitTestResult>, Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => Ok(h.hit_test_point(x, y)?),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => Ok(h.hit_test_point(x, y)?),
        }
    }

    fn hit_test_text_position(
        &self,
        text_index: usize,
        trailing: bool,
    ) -> Result<(f32, f32), Self::Error> {
        match self {
            #[cfg(feature = "gpu-render-backend")]
            Self::GPU(h) => Ok(h.hit_test_text_position(text_index, trailing)?),
            #[cfg(feature = "cpu-render-backend")]
            Self::CPU(h) => Ok(h.hit_test_text_position(text_index, trailing)?),
        }
    }
}
