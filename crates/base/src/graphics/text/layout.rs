use crate::graphics::{HitTestResult, TextChunk};
use crate::types::Rect;

mod config;
pub use config::*;
mod cosmic;
pub use cosmic::*;

pub trait LayoutText {
    type Error;
    type BrushHandle;
    type TextFormatHandle;

    fn set_font_size(&mut self, size: f32);
    fn set_default_text_format(&mut self, default_text_format: Self::TextFormatHandle);
    fn set_left(&mut self, left: f32);
    fn set_top(&mut self, top: f32);
    fn set_width(&mut self, width: f32);
    fn set_height(&mut self, height: f32);
    fn set_text(&mut self, text: String);
    fn set_chunks(
        &mut self,
        chunks: Option<Vec<TextChunk<Self::BrushHandle, Self::TextFormatHandle>>>,
    );
    fn bounds(&self) -> Rect<f32>;
    fn measure_text(&self) -> Result<(f32, f32), Self::Error>;
    fn hit_test_point(&self, x: f32, y: f32) -> Result<Option<HitTestResult>, Self::Error>;
    fn hit_test_text_position(
        &self,
        text_index: usize,
        trailing: bool,
    ) -> Result<(f32, f32), Self::Error>;
}
