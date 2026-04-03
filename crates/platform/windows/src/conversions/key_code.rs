// windows/mod.rs
use crate::base::KeyCode;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_A, VK_B, VK_BACK,
    VK_C, VK_D, VK_DELETE, VK_DOWN, VK_E, VK_ESCAPE, VK_F, VK_F1, VK_F10, VK_F11, VK_F12, VK_F2,
    VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_G, VK_H, VK_I, VK_J, VK_K, VK_L, VK_LEFT,
    VK_M, VK_N, VK_O, VK_P, VK_Q, VK_R, VK_RETURN, VK_RIGHT, VK_S, VK_SPACE, VK_T, VK_TAB, VK_U,
    VK_UP, VK_V, VK_W, VK_X, VK_Y, VK_Z,
};

pub trait FromVkCode {
    fn from_vk(vk: VIRTUAL_KEY) -> KeyCode;
}

impl FromVkCode for KeyCode {
    fn from_vk(vk: VIRTUAL_KEY) -> KeyCode {
        match vk {
            // 字母
            VK_A => KeyCode::A,
            VK_B => KeyCode::B,
            VK_C => KeyCode::C,
            VK_D => KeyCode::D,
            VK_E => KeyCode::E,
            VK_F => KeyCode::F,
            VK_G => KeyCode::G,
            VK_H => KeyCode::H,
            VK_I => KeyCode::I,
            VK_J => KeyCode::J,
            VK_K => KeyCode::K,
            VK_L => KeyCode::L,
            VK_M => KeyCode::M,
            VK_N => KeyCode::N,
            VK_O => KeyCode::O,
            VK_P => KeyCode::P,
            VK_Q => KeyCode::Q,
            VK_R => KeyCode::R,
            VK_S => KeyCode::S,
            VK_T => KeyCode::T,
            VK_U => KeyCode::U,
            VK_V => KeyCode::V,
            VK_W => KeyCode::W,
            VK_X => KeyCode::X,
            VK_Y => KeyCode::Y,
            VK_Z => KeyCode::Z,

            // 主键盘数字
            VK_0 => KeyCode::Num0,
            VK_1 => KeyCode::Num1,
            VK_2 => KeyCode::Num2,
            VK_3 => KeyCode::Num3,
            VK_4 => KeyCode::Num4,
            VK_5 => KeyCode::Num5,
            VK_6 => KeyCode::Num6,
            VK_7 => KeyCode::Num7,
            VK_8 => KeyCode::Num8,
            VK_9 => KeyCode::Num9,

            // 功能键 F1-F12
            VK_F1 => KeyCode::F1,
            VK_F2 => KeyCode::F2,
            VK_F3 => KeyCode::F3,
            VK_F4 => KeyCode::F4,
            VK_F5 => KeyCode::F5,
            VK_F6 => KeyCode::F6,
            VK_F7 => KeyCode::F7,
            VK_F8 => KeyCode::F8,
            VK_F9 => KeyCode::F9,
            VK_F10 => KeyCode::F10,
            VK_F11 => KeyCode::F11,
            VK_F12 => KeyCode::F12,

            // 常用控制键
            VK_RETURN => KeyCode::Enter,
            VK_ESCAPE => KeyCode::Escape,
            VK_TAB => KeyCode::Tab,
            VK_BACK => KeyCode::Backspace,
            VK_SPACE => KeyCode::Space,
            VK_LEFT => KeyCode::Left,
            VK_UP => KeyCode::Up,
            VK_RIGHT => KeyCode::Right,
            VK_DOWN => KeyCode::Down,
            VK_DELETE => KeyCode::Delete,

            // 其他
            VIRTUAL_KEY(v) => KeyCode::Platform(v),
        }
    }
}
