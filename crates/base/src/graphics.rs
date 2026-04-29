//! flor graphics layer
//!
//! This crate provides a cross-platform rendering framework, supporting CPU and GPU backends.
//! Features include:
//! - Unified RenderBase trait
//! - TextFormat and Brush abstractions
//! - Shadow, blur, rounded corners, and gradient support
//! - Cross-platform CPU/GPU rendering

mod draw_options;
mod error;
mod gradient;
mod handle;
mod path;
mod render;
mod scale_mode;
mod shadow;
mod text;

pub use {
    draw_options::*, error::*, gradient::*, handle::*, path::*, render::*, scale_mode::*,
    shadow::*, text::*,
};
