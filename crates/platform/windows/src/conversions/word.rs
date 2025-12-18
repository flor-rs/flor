#[inline(always)]
pub(crate) const fn loword(x: u32) -> u16 {
    (x & 0xFFFF) as u16
}

#[inline(always)]
pub(crate) const fn hiword(x: u32) -> u16 {
    ((x >> 16) & 0xFFFF) as u16
}
