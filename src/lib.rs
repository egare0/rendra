//! `rendra` is a high-level, ergonomic rendering abstraction over wgpu.

mod error;
mod renderer;
mod color;

pub mod raw;

pub use color::Color;
pub use renderer::{Renderer, RendererBuilder, Frame};
pub use error::RendraError;

/// Shortcut for `Renderer::builder()`.
#[inline]
#[must_use]
pub fn builder() -> RendererBuilder {
    Renderer::builder()
}