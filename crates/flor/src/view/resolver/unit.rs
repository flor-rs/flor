mod metrics;
mod resolver;

pub use {metrics::*, resolver::*};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum Unit {
    #[default]
    Px,
    Pt,

    Rem,

    Vw,
    Vh,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Length {
    Px(f32),
    Pt(f32),

    Rem(f32),

    Vw(f32),
    Vh(f32),
}
