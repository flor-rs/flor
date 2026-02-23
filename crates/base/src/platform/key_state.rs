#[derive(Copy, Clone, Debug)]
pub struct KeyState {
    pub control_is_down: bool,
    pub lbutton_is_down: bool,
    pub mbutton_is_down: bool,
    pub rbutton_is_down: bool,
    pub shift_is_down: bool,
    pub x_button1_is_down: bool,
    pub x_button2_is_down: bool,
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            control_is_down: false,
            lbutton_is_down: false,
            mbutton_is_down: false,
            rbutton_is_down: false,
            shift_is_down: false,
            x_button1_is_down: false,
            x_button2_is_down: false,
        }
    }
}
