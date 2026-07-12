//! Core rendering types: [`Device`], [`Surface`], [`Renderer`] and [`Frame`].
//!
//! This module is always compiled, regardless of which feature flags are
//! enabled. `render2d` and `render3d` build on top of it.

mod device;
mod surface;
mod renderer;
mod error;
mod color;
mod texture;
mod layouts;

pub use device::Device;
pub use surface::Surface;
pub use renderer::{Frame, Renderer};
pub(crate) use renderer::{LightsRaw, DRAW_CAPACITY};
pub(crate) use layouts::{frame_globals_layout, per_draw_layout};
pub use error::RendraError;
pub use color::Color;
pub use texture::{Texture, TextureBuilder, Filter};