use flor_base::graphics::ImageHandle;

#[derive(Debug, Clone)]
pub struct GlImageHandle {}

impl ImageHandle for GlImageHandle {
    fn frame_count(&self) -> usize {
        0
    }

    fn delays(&self) -> &[u16] {
        &[0]
    }

    fn total_delays(&self) -> u128 {
        0
    }

    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn get_width(&self) -> u32 {
        0
    }

    fn get_height(&self) -> u32 {
        0
    }
}
