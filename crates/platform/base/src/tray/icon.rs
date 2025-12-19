#[derive(Debug, Clone)]
pub enum IconSource {
    /// 方便路径：直接加载磁盘文件（仅支持 .ico，因为 LoadImageW 只认这个）
    Path(std::path::PathBuf),

    /// 万能接口：原始 RGBA 像素数据
    /// 用户可以用 image crate 解码 png，或者自己算出这些字节
    /// 你的框架只负责把这一坨 bytes 喂给系统 API
    Raw {
        width: u32,
        height: u32,
        rgba_data: Vec<u8>, // 必须是 R-G-B-A 顺序
    },
}

impl IconSource {
    #[inline]
    pub fn from_path(path: std::path::PathBuf) -> IconSource {
        IconSource::Path(path)
    }

    #[inline]
    pub fn from_raw(width: u32, height: u32, rgba_data: Vec<u8>) -> IconSource {
        IconSource::Raw {
            width,
            height,
            rgba_data,
        }
    }
}
