use flor_base::platform::KeyState;
use windows::Win32::System::SystemServices::{MK_CONTROL, MK_LBUTTON, MK_MBUTTON, MK_RBUTTON, MK_SHIFT, MK_XBUTTON1, MK_XBUTTON2, MODIFIERKEYS_FLAGS};

pub trait IntoKeyState {
    fn into_key_state(self) -> KeyState;
}

impl IntoKeyState for MODIFIERKEYS_FLAGS {
    #[inline]
    fn into_key_state(self) -> KeyState {
        KeyState {
            control_is_down: self.contains(MK_CONTROL),
            lbutton_is_down: self.contains(MK_LBUTTON),
            mbutton_is_down: self.contains(MK_MBUTTON),
            rbutton_is_down: self.contains(MK_RBUTTON),
            shift_is_down: self.contains(MK_SHIFT),
            x_button1_is_down: self.contains(MK_XBUTTON1),
            x_button2_is_down: self.contains(MK_XBUTTON2),
        }
    }
}
