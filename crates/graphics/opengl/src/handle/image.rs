use flor_base::graphics::ImageHandle;
use glow::NativeTexture;
use image::codecs::gif::GifDecoder;
use image::codecs::webp::WebPDecoder;
use image::{AnimationDecoder, Frame, ImageFormat, ImageResult};
use parking_lot::Mutex;
use std::io::{BufReader, Cursor};
use std::sync::Arc;

#[derive(Debug)]
pub struct GlImageHandle {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub delays: Vec<u16>,
    pub total_delays: u128,
    // 暂时先用 raw bytes 或者 CPU 端的副本保存。由于 OpenGL Texture 无法实现 Send/Sync 且与 GL Context 深度绑定，
    // 我们可能需要在初次使用该 ImageHandle 时由 Renderer 上传，或者在这里保存 texture 的内部 ID 和 context 相关封装。
    // 为了满足目前的句柄拷贝语义，我们存放预乘后的像素数据，由 Renderer 维护 ID 映射。
    pub frames: Arc<Vec<Vec<u8>>>,
    pub cache: Mutex<Vec<NativeTexture>>,
}

impl Clone for GlImageHandle {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            width: self.width,
            height: self.height,
            delays: self.delays.clone(),
            total_delays: self.total_delays,
            frames: self.frames.clone(),
            cache: Mutex::new(Vec::new()),
        }
    }
}

impl ImageHandle for GlImageHandle {
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

impl GlImageHandle {
    pub fn new(id: u64, bytes: &[u8]) -> ImageResult<Self> {
        let format = image::guess_format(bytes)?;

        match format {
            ImageFormat::Gif => {
                let decoder = GifDecoder::new(BufReader::new(Cursor::new(bytes)))?;
                Self::build_handle(id, decoder.into_frames().collect_frames()?)
            }
            ImageFormat::WebP => {
                let mut decoder = WebPDecoder::new(BufReader::new(Cursor::new(bytes)))?;
                if decoder.has_animation() {
                    Self::build_handle(id, decoder.into_frames().collect_frames()?)
                } else {
                    let img = image::load_from_memory_with_format(bytes, format)?.into_rgba8();
                    Ok(Self {
                        id,
                        width: img.width(),
                        height: img.height(),
                        delays: vec![],
                        total_delays: 0,
                        frames: Arc::new(vec![img.into_raw()]),
                        cache: Mutex::new(Vec::new()),
                    })
                }
            }
            _ => {
                let img = image::load_from_memory_with_format(bytes, format)?.into_rgba8();
                Ok(Self {
                    id,
                    width: img.width(),
                    height: img.height(),
                    delays: vec![],
                    total_delays: 0,
                    frames: Arc::new(vec![img.into_raw()]),
                    cache: Mutex::new(Vec::new()),
                })
            }
        }
    }

    fn build_handle(id: u64, collect_frames: Vec<Frame>) -> ImageResult<GlImageHandle> {
        let mut delays = vec![];
        let mut frames = vec![];
        let mut width = 0;
        let mut height = 0;
        for (i, frame) in collect_frames.into_iter().enumerate() {
            if i == 0 {
                width = frame.buffer().width();
                height = frame.buffer().height();
            }
            {
                let delay = frame.delay();
                let (numer, denom) = delay.numer_denom_ms();

                delays.push((if denom == 0 { 0 } else { numer / denom }) as u16);
            }
            frames.push(frame.into_buffer().into_raw());
        }
        let total_delays = delays.iter().map(|delay| *delay as u128).sum();
        Ok(Self {
            id,
            width,
            height,
            delays,
            total_delays,
            frames: Arc::new(frames.clone()), // Use cloned frames to satisfy scope/len
            cache: Mutex::new(Vec::new()),
        })
    }
}
