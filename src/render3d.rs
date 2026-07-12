//! 3D rendering: meshes, materials, viewports and lighting.
//!
//! Everything here requires the `3d` feature, which is enabled by default.

mod mesh;
mod shader;

pub use mesh::{Mesh, MeshBuilder, Vertex};
pub use shader::{Shader, ShaderBuilder};