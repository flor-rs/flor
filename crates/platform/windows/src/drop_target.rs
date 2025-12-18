use crate::conversions::drop_effect::ToWinDropEffect;
use crate::conversions::key_state::IntoKeyState;
use crate::{proc, WindowId};
use flor_platform_base::{DragData, DragFormat, DropEffect, Message, MousePosition};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use windows::core::implement;
use windows::Win32::Foundation::{POINT, POINTL};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::Com::FORMATETC;
use windows::Win32::System::Com::{IDataObject, DATADIR_GET};
use windows::Win32::System::Com::{DVASPECT_CONTENT, TYMED_HGLOBAL};
use windows::Win32::System::DataExchange::GetClipboardFormatNameW;
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::CF_HDROP;
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, CF_DIB, CF_TEXT, CF_UNICODETEXT, CLIPBOARD_FORMAT, DROPEFFECT,
    DROPEFFECT_NONE,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
use windows_core::{Error, Ref};

static DROP_TARGET_MAP: Lazy<RwLock<FxHashMap<WindowId, DropTarget>>> =
    Lazy::new(|| RwLock::new(FxHashMap::default()));

#[implement(IDropTarget)]
pub struct DropTarget {
    window_id: WindowId,
    cached_formats: RwLock<Vec<DragFormat>>,
}

impl DropTarget {
    pub fn new(window_id: WindowId) {
        let dt = DropTarget {
            window_id,
            cached_formats: RwLock::new(vec![]),
        };
        DROP_TARGET_MAP.write().insert(window_id, dt);
    }
}

impl IDropTarget_Impl for DropTarget_Impl {
    #[allow(non_snake_case)]
    fn DragEnter(
        &self,
        p_data_obj: Ref<'_, IDataObject>,
        grf_key_state: MODIFIERKEYS_FLAGS,
        pt: &POINTL,
        p_dw_effect: *mut DROPEFFECT,
    ) -> windows_core::Result<()> {
        unsafe {
            // 1. 默认设为 None (拒绝)，等待业务层修改
            p_dw_effect.write(DROPEFFECT_NONE);

            // 2. 坐标转换 (Screen -> Client)
            let mut screen_pt = POINT { x: pt.x, y: pt.y };
            let _ = ScreenToClient(self.window_id.hwnd(), &mut screen_pt);

            // 3. 简单的格式检查 (此时不要读取文件内容，太慢)
            // 这里用辅助函数检查是否包含文件
            *self.cached_formats.write() = get_available_formats(&p_data_obj.as_ref())?;

            let formats = self.cached_formats.read();

            // 4. 构造默认的反馈
            let mut effect = DropEffect::None;

            // 5. 发送同步消息
            // 注意：这里假设 window_proc 是同步调用的
            proc().window_proc(
                self.window_id,
                Message::DragEnter {
                    key_state: grf_key_state.into_key_state(),
                    mouse_position: MousePosition {
                        x: screen_pt.x,
                        y: screen_pt.y,
                    },
                    formats: formats.as_slice(),
                    effect: &mut effect, // 传引用进去
                },
            );

            // 6. 将业务层的决定写回 Windows
            p_dw_effect.write(effect.to_win32());
        }
        Ok(())
    }

    #[allow(non_snake_case)]
    fn DragOver(
        &self,
        grf_key_state: MODIFIERKEYS_FLAGS,
        pt: &POINTL,
        p_dw_effect: *mut DROPEFFECT,
    ) -> windows_core::Result<()> {
        unsafe {
            *p_dw_effect = DROPEFFECT_NONE;

            let mut screen_pt = POINT { x: pt.x, y: pt.y };
            let _ = ScreenToClient(self.window_id.hwnd(), &mut screen_pt);

            let mut effect = DropEffect::None;

            // [关键] 从缓存中借用格式列表
            let formats = self.cached_formats.read();

            // Windows 会频繁调用 DragOver，所以这里不要做耗时操作
            proc().window_proc(
                self.window_id,
                Message::DragOver {
                    key_state: grf_key_state.into_key_state(),
                    mouse_position: MousePosition {
                        x: screen_pt.x,
                        y: screen_pt.y,
                    },
                    formats: formats.as_slice(),
                    effect: &mut effect,
                },
            );

            *p_dw_effect = effect.to_win32();
        }
        Ok(())
    }

    #[allow(non_snake_case)]
    fn DragLeave(&self) -> windows_core::Result<()> {
        proc().window_proc(self.window_id, Message::DragLeave);
        Ok(())
    }

    #[allow(non_snake_case)]
    fn Drop(
        &self,
        p_data_obj: Ref<'_, IDataObject>,
        grf_key_state: MODIFIERKEYS_FLAGS,
        pt: &POINTL,
        p_dw_effect: *mut DROPEFFECT,
    ) -> windows_core::Result<()> {
        unsafe {
            *p_dw_effect = DROPEFFECT_NONE;

            let mut screen_pt = POINT { x: pt.x, y: pt.y };
            let _ = ScreenToClient(self.window_id.hwnd(), &mut screen_pt);

            // 1. [核心] 只有在 Drop 的时候才真正解析数据
            let data = parse_data_object(&p_data_obj.as_ref())?;

            let mut effect = DropEffect::None;

            proc().window_proc(
                self.window_id,
                Message::Drop {
                    key_state: grf_key_state.into_key_state(),
                    mouse_position: MousePosition {
                        x: screen_pt.x,
                        y: screen_pt.y,
                    },
                    data,
                    effect: &mut effect,
                },
            );

            // 如果业务层接受了，通常 Drop 成功返回 COPY 或 MOVE
            *p_dw_effect = effect.to_win32();
        }
        Ok(())
    }
}

// 辅助函数：获取剪贴板格式的名称
unsafe fn get_format_name(cf: u16) -> String {
    // 1. 处理标准格式 (Pre-defined)
    match cf {
        1 => return "CF_TEXT".to_string(),
        2 => return "CF_BITMAP".to_string(),
        8 => return "CF_DIB".to_string(),
        13 => return "CF_UNICODETEXT".to_string(),
        15 => return "CF_HDROP".to_string(),
        _ => {}
    }

    // 2. 处理注册格式 (Registered Formats)
    let mut buffer = [0u16; 256];
    let len = GetClipboardFormatNameW(cf as u32, &mut buffer);
    if len > 0 {
        String::from_utf16_lossy(&buffer[..len as usize])
    } else {
        format!("Unknown({})", cf)
    }
}

unsafe fn get_available_formats(data_obj: &Option<&IDataObject>) -> Result<Vec<DragFormat>, Error> {
    let mut formats = Vec::new();
    let Some(obj) = data_obj else {
        return Ok(formats);
    };

    // 创建枚举器来遍历所有支持的格式
    let enum_fmt = obj.EnumFormatEtc(DATADIR_GET.0 as u32)?;

    let mut fmt = [FORMATETC::default(); 1];
    let mut fetched = 0;

    // 循环获取所有格式
    while enum_fmt.Next(&mut fmt, Some(&mut fetched)).is_ok() && fetched == 1 {
        let cf = fmt[0].cfFormat;
        let name = get_format_name(cf);

        let cf = CLIPBOARD_FORMAT(cf);

        let drag_fmt = match cf {
            CF_HDROP => DragFormat::Files(name),
            CF_UNICODETEXT | CF_TEXT => DragFormat::Text(name),
            CF_DIB => DragFormat::Image(name),
            _ => DragFormat::Custom(name),
        };

        formats.push(drag_fmt);
    }

    Ok(formats)
}

// 解析真实数据
// 策略：按优先级 文件 > 文本 > 图片 尝试获取数据
// 如果你需要同时获取所有数据，可以修改返回类型为 Vec<DragData>
unsafe fn parse_data_object(data_obj: &Option<&IDataObject>) -> Result<DragData, Error> {
    let Some(obj) = data_obj else {
        return Ok(DragData::None);
    };

    // =================================================================================
    // 1. 尝试解析文件 (CF_HDROP)
    // =================================================================================
    let fmt_file = FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };

    if let Ok(medium) = obj.GetData(&fmt_file) {
        // windows crate 的 STGMEDIUM 会在 Drop 时自动释放，但我们需要手动 Lock/Unlock

        let ptr = GlobalLock(medium.u.hGlobal);
        if !ptr.is_null() {
            // 使用 finally 模式或确保在所有路径都 Unlock，这里简单处理
            let h_drop = HDROP(ptr);
            let count = DragQueryFileW(h_drop, 0xFFFFFFFF, None);
            let mut files = Vec::new();

            for i in 0..count {
                let len = DragQueryFileW(h_drop, i, None);
                if len > 0 {
                    let mut buffer = vec![0u16; (len + 1) as usize];
                    DragQueryFileW(h_drop, i, Some(&mut buffer));
                    // 去掉末尾的 \0
                    let s = String::from_utf16_lossy(&buffer[..len as usize]);
                    files.push(PathBuf::from(s));
                }
            }
            GlobalUnlock(medium.u.hGlobal)?;
            return Ok(DragData::Files(files));
        }
    }

    // =================================================================================
    // 2. 尝试解析文本 (CF_UNICODETEXT)
    // =================================================================================
    let fmt_text = FORMATETC {
        cfFormat: CF_UNICODETEXT.0, // 优先使用 Unicode
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };

    if let Ok(medium) = obj.GetData(&fmt_text) {
        let ptr = GlobalLock(medium.u.hGlobal);
        if !ptr.is_null() {
            let ptr_u16 = ptr as *const u16;
            // 计算字符串长度 (寻找 \0)
            let mut len = 0;
            while *ptr_u16.offset(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr_u16, len as usize);
            let text = String::from_utf16_lossy(slice);

            GlobalUnlock(medium.u.hGlobal)?;
            return Ok(DragData::Text(text));
        }
    }

    // =================================================================================
    // 3. 尝试解析位图数据 (CF_DIB)
    // =================================================================================
    let fmt_dib = FORMATETC {
        cfFormat: CF_DIB.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };

    if let Ok(medium) = obj.GetData(&fmt_dib) {
        let ptr = GlobalLock(medium.u.hGlobal);
        if !ptr.is_null() {
            let size = GlobalSize(medium.u.hGlobal);
            if size > 0 {
                let slice = std::slice::from_raw_parts(ptr as *const u8, size);
                let data = slice.to_vec(); // 拷贝数据

                GlobalUnlock(medium.u.hGlobal)?;
                return Ok(DragData::Image(data));
            }
            GlobalUnlock(medium.u.hGlobal)?;
        }
    }

    Ok(DragData::None)
}
