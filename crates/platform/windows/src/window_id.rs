use windows::Win32::Foundation::HWND;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WindowId(pub isize);

impl WindowId {
    #[inline]
    pub fn hwnd(&self) -> HWND {
        HWND(self.0 as *mut _)
    }
}

impl Default for WindowId {
    fn default() -> Self {
        Self(0)
    }
}

impl From<isize> for WindowId {
    fn from(value: isize) -> Self {
        WindowId(value)
    }
}

impl From<HWND> for WindowId {
    fn from(value: HWND) -> Self {
        WindowId(value.0 as isize)
    }
}

impl Into<HWND> for WindowId {
    fn into(self) -> HWND {
        HWND(self.0 as *mut _)
    }
}

impl Into<i32> for WindowId {
    fn into(self) -> i32 {
        self.0 as i32
    }
}
