//! `rendra` is a high-level, ergonomic rendering abstraction over wgpu.

mod error;
mod renderer;

pub mod raw;

pub use renderer::{Renderer, RendererBuilder};
pub use error::RendraError;

/// Shortcut for `Renderer::builder()`.
#[inline]
#[must_use]
pub fn builder() -> RendererBuilder {
    Renderer::builder()
}