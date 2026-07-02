//! Core rendering types: [`Device`], [`Surface`], [`Renderer`] and [`Frame`].
//!
//! This module is always compiled, regardless of which feature flags are
//! enabled. `render2d` and `render3d` build on top of it.

mod device;
mod surface;
mod renderer;
mod error;
mod color;

pub mod raw;

pub use device::Device;
pub use surface::Surface;
pub use renderer::{Frame, Renderer};
pub use error::RendraError;
pub use color::Color;