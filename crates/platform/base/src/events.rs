#[cfg(feature = "drag-drop")]
use crate::drag_drop::{DragFormat, DropEffect};
use crate::key_code::KeyCode;
use crate::key_state::KeyState;
use crate::mouse_position::MousePosition;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::path::PathBuf;

pub trait EventMsg: Any + Debug {}

impl dyn EventMsg {
    pub fn is<T: Any>(&self) -> bool {
        // Get `TypeId` of the type this function is instantiated with.
        let t = TypeId::of::<T>();

        // Get `TypeId` of the type in the trait object (`self`).
        let concrete = self.type_id();

        // Compare both `TypeId`s on equality.
        t == concrete
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no Other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&*(self as *const dyn EventMsg as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented Any for all types; no Other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(&mut *(self as *mut dyn EventMsg as *mut T)) }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    /// 正在组合中 (Preedit)。
    /// 必须用 String，因为拼音可能很长，且长度不定，堆分配不可避免。
    /// 但 IME 事件频率远低于 CPU 处理速度，这里不是瓶颈。
    ImeIng(String),

    /// 组合结束 (Result) 或 粘贴文本。
    /// 同样必须用 String。
    ImeEnd(String),

    /// 普通字符输入 (WM_CHAR)。
    /// ✅ 性能关键点：直接在栈上存储 char (4字节)，无堆内存分配 (No Heap Alloc)。
    Char(char),

    /// 控制字符 (如 Backspace \x08, Enter \r)。
    /// 同样无堆分配。
    Control(char),
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
}
#[derive(Debug)]
pub enum DragData {
    None,
    Files(Vec<PathBuf>),
    Text(String),
    Image(Vec<u8>),
    // 尚未读取的原始句柄（用于延迟读取）
    Raw(Box<dyn Any>),
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ScrollAxis {
    Vertical,
    Horizontal,
}
#[derive(Debug)]
pub enum Message<'a> {
    WindowDestroy,
    ImeStart,
    ImeInput(InputEvent),
    ImeEnd,
    CaptureChange,
    CloseRequested {
        prevent: &'a mut bool,
    },
    Draw,
    /// 传递时，每个组件收到的是自己的可用宽高
    Resize {
        width: u32,
        height: u32,
    },
    Focus,
    Blur,
    MouseMove {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    LButtonDoubleClick {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    LButtonDown {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    LButtonUp {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    RButtonDoubleClick {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    RButtonDown {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    RButtonUp {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    MButtonDoubleClick {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    MButtonDown {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    MButtonUp {
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    KeyDown {
        code: KeyCode,
        is_alt: bool,
        is_ctrl: bool,
        is_shift: bool,
    },
    KeyUp {
        code: KeyCode,
        is_alt: bool,
        is_ctrl: bool,
        is_shift: bool,
    },
    MouseWheel {
        axis: ScrollAxis,
        delta: f32,
        key_state: KeyState,
        mouse_position: MousePosition,
    },
    #[cfg(feature = "theme-change")]
    ThemeChanged(ThemeMode), // 需要重新检测深色模式
    WorkAreaChanged,           // 任务栏/分辨率改变
    WheelSettingsChanged(u32), // 滚轮行数改变 (携带新值)
    MouseLeave,
    DpiChange {
        dpi_x: f64,
        dpi_y: f64,
    },
    Cursor,
    #[cfg(feature = "drag-drop")]
    // 拖拽进入
    DragEnter {
        key_state: KeyState,
        mouse_position: MousePosition,
        formats: &'a [DragFormat], // 支持的格式 (如 "File", "Text")
        effect: &'a mut DropEffect,
    },
    #[cfg(feature = "drag-drop")]
    // 拖拽悬停
    DragOver {
        key_state: KeyState,
        mouse_position: MousePosition,
        formats: &'a [DragFormat], // 支持的格式 (如 "File", "Text")
        effect: &'a mut DropEffect,
    },
    #[cfg(feature = "drag-drop")]
    // 拖拽离开
    DragLeave,
    #[cfg(feature = "drag-drop")]
    // 放置 (真正的数据交换)
    Drop {
        key_state: KeyState,
        mouse_position: MousePosition,
        data: DragData, // 解析好的数据
        effect: &'a mut DropEffect,
    },
}

impl Message<'_> {
    // pub fn get_event_type(&self) -> EventType {
    //     match self {
    //         Message::WindowDestroy => EventType::WindowDestroy,
    //         Message::Resize { .. } => EventType::Resize,
    //         Message::Focus => EventType::Focus,
    //         Message::Blur => EventType::Blur,
    //         // EventMessage::LeftClick => { EventType::LeftClick }
    //         // EventMessage::LeftDoubleClick => { EventType::LeftDoubleClick }
    //         // EventMessage::MiddleClick => { EventType::MiddleClick }
    //         // EventMessage::MiddleDoubleClick => { EventType::MiddleDoubleClick }
    //         // EventMessage::RightClick => { EventType::RightClick }
    //         // EventMessage::RightDoubleClick => { EventType::RightDoubleClick }
    //         _ => EventType::Blur,
    //     }
    // }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EventType {
    WindowDestroy,
    Resize,
    Focus,
    Blur,
    LeftClick,
    LeftDoubleClick,
    MiddleClick,
    MiddleDoubleClick,
    RightClick,
    RightDoubleClick,
}
