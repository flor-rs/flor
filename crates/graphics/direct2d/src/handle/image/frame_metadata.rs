use windows::core::{w, Error};
use windows::Win32::Graphics::Imaging::IWICMetadataQueryReader;
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PROPVARIANT};

#[derive(Clone, Copy, Debug)]
pub struct FrameMetadata {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    // 0: None, 1: Keep, 2: Background, 3: Previous
    pub disposal: u8,
    pub delay: u16,
}

impl FrameMetadata {
    pub fn default() -> Self {
        Self {
            left: 0.,
            top: 0.,
            width: 0.,
            height: 0.,
            disposal: 1,
            delay: 0,
        }
    }

    // 辅助函数：读取帧元数据
    pub(crate) fn get_frame_metadata(
        reader: &IWICMetadataQueryReader,
    ) -> Result<FrameMetadata, Error> {
        let mut meta = FrameMetadata::default();

        unsafe {
            // 读取偏移量和尺寸
            let mut prop = PROPVARIANT::default();
            reader.GetMetadataByName(w!("/imgdesc/Left"), &mut prop)?;
            meta.left = prop.Anonymous.Anonymous.Anonymous.uiVal as f32;
            PropVariantClear(&mut prop)?;

            let mut prop = PROPVARIANT::default();
            reader.GetMetadataByName(w!("/imgdesc/Top"), &mut prop)?;

            meta.top = prop.Anonymous.Anonymous.Anonymous.uiVal as f32;
            PropVariantClear(&mut prop)?;

            let mut prop = PROPVARIANT::default();
            reader.GetMetadataByName(w!("/imgdesc/Width"), &mut prop)?;
            meta.width = prop.Anonymous.Anonymous.Anonymous.uiVal as f32;
            PropVariantClear(&mut prop)?;

            let mut prop = PROPVARIANT::default();
            reader.GetMetadataByName(w!("/imgdesc/Height"), &mut prop)?;
            meta.height = prop.Anonymous.Anonymous.Anonymous.uiVal as f32;
            PropVariantClear(&mut prop)?;

            // 读取处置方法 (Disposal Method)
            // 0: Unspecified (Treat as Keep), 1: Do Not Dispose (Keep),
            // 2: Restore to Background, 3: Restore to Previous
            let mut prop = PROPVARIANT::default();
            reader.GetMetadataByName(w!("/grctlext/Disposal"), &mut prop)?;
            meta.disposal = prop.Anonymous.Anonymous.Anonymous.bVal;
            PropVariantClear(&mut prop)?;

            let mut prop = PROPVARIANT::default();
            reader.GetMetadataByName(w!("/grctlext/Delay"), &mut prop)?;
            meta.delay = (prop.Anonymous.Anonymous.Anonymous.uiVal * 10).max(20);
            PropVariantClear(&mut prop)?;
        }
        Ok(meta)
    }
}
