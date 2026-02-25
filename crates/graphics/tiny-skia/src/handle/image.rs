use flor_base::graphics::ImageHandle;

#[derive(Clone)]
pub struct TinySkiaImageHandle {
    pub width: u32,
    pub height: u32,
    pub delays: Vec<u16>,
    pub total_delays: u128,
    pub frames: std::sync::Arc<Vec<tiny_skia::Pixmap>>,
}

impl std::fmt::Debug for TinySkiaImageHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TinySkiaImageHandle")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("frames_count", &self.frames.len())
            .finish()
    }
}

impl PartialEq for TinySkiaImageHandle {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.frames, &other.frames)
    }
}

impl Eq for TinySkiaImageHandle {}

impl ImageHandle for TinySkiaImageHandle {
    fn frame_count(&self) -> usize {
        self.delays.len().max(1)
    }

    fn delays(&self) -> &[u16] {
        &self.delays
    }

    fn total_delays(&self) -> u128 {
        self.total_delays
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }
}
