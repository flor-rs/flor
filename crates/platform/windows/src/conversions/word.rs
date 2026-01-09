#[inline(always)]
pub(crate) const fn loword_u16(x: u32) -> u16 {
    (x & 0xFFFF) as u16
}

#[inline(always)]
pub(crate) const fn hiword_u16(x: u32) -> u16 {
    ((x >> 16) & 0xFFFF) as u16
}

#[inline(always)]
pub(crate) const fn hiword_i16(x: u32) -> i16 {
    (x >> 16) as i16
}
