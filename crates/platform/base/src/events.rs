use crate::drop_effect::DropEffect;
use crate::key_code::KeyCode;
use crate::key_state::KeyState;
use crate::mouse_position::MousePosition;
use std::any::{Any, TypeId};
use std::fmt::Debug;

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

#[derive(Debug)]
pub enum Message<'a> {
    WindowDestroy,
    Close,
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
    MouseLeave,
    DpiChange {
        dpi_x: f32,
        dpi_y: f32,
    },
    DragEnter,
    DragOver {
        key_state: KeyState,
        mouse_position: MousePosition,
        drop_effect: &'a mut DropEffect,
    },
    DragLeave,
    Drop,
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
