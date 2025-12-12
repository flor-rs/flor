#[derive(Debug, Copy, Clone)]
pub enum DeviceKind {
    Desktop,
    Tablet,
    Mobile,
}

impl DeviceKind {
    // todo 改成底层提供
    #[inline]
    pub fn simple_detect() -> DeviceKind {
        #[cfg(target_os = "windows")]
        {
            DeviceKind::Desktop
        }
        #[cfg(target_os = "macos")]
        {
            DeviceKind::Desktop
        }
        #[cfg(target_os = "linux")]
        {
            DeviceKind::Desktop
        }
        #[cfg(target_os = "android")]
        {
            DeviceKind::Mobile
        }
        #[cfg(target_os = "ios")]
        {
            DeviceKind::Mobile
        }
    }
}
