use crate::conversions::encode_wide::encode_wide;
use crate::Error;
use flor_platform_base::CursorIcon;
use windows::Win32::Graphics::Gdi::CreateBitmap;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, DestroyCursor, LoadCursorW, LoadImageW, HCURSOR, ICONINFO, IDC_APPSTARTING,
    IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_HELP, IDC_IBEAM, IDC_NO, IDC_SIZEALL, IDC_SIZENESW,
    IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, IDC_WAIT, IMAGE_CURSOR, LR_DEFAULTSIZE, LR_LOADFROMFILE,
};
use windows_core::{BOOL, PCWSTR};

#[derive(Debug, Clone)]
pub struct Cursor {
    pub inner: HCURSOR,
    owned: bool,
}

unsafe impl Send for Cursor {}
unsafe impl Sync for Cursor {}

impl flor_platform_base::CursorHandle for Cursor {
    type Handle = HCURSOR;
    type Error = Error;

    fn load_from_system(cursor_icon: CursorIcon) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let icon = match cursor_icon {
            CursorIcon::Default => IDC_ARROW,
            CursorIcon::Text => IDC_IBEAM,
            CursorIcon::Pointer => IDC_HAND,
            CursorIcon::Wait => IDC_WAIT,
            CursorIcon::Progress => IDC_APPSTARTING, // Windows 没有明确的 Progress，退回到 Default
            CursorIcon::Crosshair => IDC_CROSS,
            CursorIcon::NotAllowed => IDC_NO,

            // 移动/抓取
            CursorIcon::Move | CursorIcon::Grab | CursorIcon::Grabbing => IDC_SIZEALL,

            // 调整大小 (NS 和 EW)
            CursorIcon::NResize | CursorIcon::SResize | CursorIcon::NsResize => IDC_SIZENS,
            CursorIcon::EResize | CursorIcon::WResize | CursorIcon::EwResize => IDC_SIZEWE,

            // 调整大小 (斜向)
            CursorIcon::NwResize | CursorIcon::SeResize | CursorIcon::NwseResize => IDC_SIZENWSE,
            CursorIcon::NeResize | CursorIcon::SwResize | CursorIcon::NeswResize => IDC_SIZENESW,

            // 其他
            CursorIcon::Help => IDC_HELP,
            CursorIcon::Cell => IDC_HELP,
            // Alias 和 Copy 通常在拖放操作中自动处理，或使用自定义光标。这里退回到 Default。
            CursorIcon::Alias | CursorIcon::Copy | CursorIcon::ContextMenu => IDC_ARROW,
        };

        // 1. 使用 Win32 ID 加载光标资源。
        // 第一个参数 (0) 表示加载系统预定义光标。
        unsafe {
            Ok(Self {
                inner: LoadCursorW(None, icon)?,
                owned: false,
            })
        }
    }

    fn from_file_path(path: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let wide_path = encode_wide(path);

        unsafe {
            // LoadImageW 是比 LoadCursorW 更现代、更强大的 API
            let handle = LoadImageW(
                None, // 文件加载时必须为 None
                PCWSTR(wide_path.as_ptr()),
                IMAGE_CURSOR, // 告诉它我们要加载的是光标
                0,
                0, // 0,0 表示使用文件的默认尺寸
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            )?;

            Ok(Self {
                inner: HCURSOR(handle.0),
                owned: true,
            })
        }
    }

    fn from_rgba_bytes(
        pixels: &[u8],
        width: u32,
        height: u32,
        hot_x: u32,
        hot_y: u32,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        unsafe {
            // 1. 创建颜色位图
            // 注意：这里为了演示使用了 CreateBitmap。
            // 对于完美的 Alpha 透明支持，标准做法是使用 CreateDIBSection。
            // 但如果只是简单的掩码光标，这样通常能工作。
            // 严谨的 GUI 框架通常会在这里做 BGRA 转换和 DIBSection 创建。
            let hbm_color = CreateBitmap(
                width as i32,
                height as i32,
                1,
                32,
                Some(pixels.as_ptr() as *const _),
            );

            // 2. 创建掩码位图 (即使是全彩光标也需要这个句柄存在)
            let hbm_mask = CreateBitmap(width as i32, height as i32, 1, 1, None);

            // 3. 填充 ICONINFO
            let mut icon_info = ICONINFO {
                fIcon: BOOL::from(false), // FALSE 表示它是光标 (Cursor)，TRUE 表示图标 (Icon)
                xHotspot: hot_x,
                yHotspot: hot_y,
                hbmMask: hbm_mask,
                hbmColor: hbm_color,
            };

            // 4. 创建光标
            let handle = CreateIconIndirect(&mut icon_info)?;

            Ok(Self {
                inner: HCURSOR(handle.0),
                owned: true, // 这是我们创建的，必须我们销毁
            })
        }
    }

    fn handle(&self) -> Self::Handle {
        self.inner
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        if self.owned && !self.inner.is_invalid() {
            unsafe {
                // DestroyCursor 实际上参数是 HCURSOR，但在 Windows API绑定中有时混用
                let _ = DestroyCursor(self.inner);
            }
        }
    }
}
