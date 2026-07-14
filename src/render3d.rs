//! 3D rendering: meshes, materials, viewports and lighting.
//!
//! Everything here requires the `3d` feature, which is enabled by default.

mod material;
mod mesh;
mod shader;
mod viewport;
mod draw;

pub use material::{Material, MaterialBuilder};
pub use mesh::{Mesh, MeshBuilder, Vertex};
pub use shader::{Shader, ShaderBuilder};
pub use viewport::Viewport3D;