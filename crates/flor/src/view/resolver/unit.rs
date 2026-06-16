mod metrics;
mod resolver;

pub use {metrics::*, resolver::*};

/// Length units used by resolver-backed layout values.
///
/// All units are resolved to pixels before being passed to layout. `Rem` uses
/// the current window's root-em size (`WindowOption::rem_px`, default 16.0).
/// `Pt` uses the current window DPI: 1pt = dpi / 72px.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum Unit {
    /// Pixels. No conversion is applied.
    #[default]
    Px,

    /// Points. Converted with the current window DPI.
    Pt,

    /// Root em. 1rem equals the current window's configured rem size in pixels.
    Rem,

    /// Viewport width unit. 1vw equals 1% of the current client-area width.
    Vw,

    /// Viewport height unit. 1vh equals 1% of the current client-area height.
    Vh,
}

/// A numeric length value tagged with a unit.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Length {
    /// Pixels. No conversion is applied.
    Px(f32),

    /// Points. Converted with the current window DPI.
    Pt(f32),

    /// Root em. 1rem equals the current window's configured rem size in pixels.
    Rem(f32),

    /// Viewport width unit. 1vw equals 1% of the current client-area width.
    Vw(f32),

    /// Viewport height unit. 1vh equals 1% of the current client-area height.
    Vh(f32),
}
