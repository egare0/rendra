//! 3D rendering: meshes, materials, viewports and lighting.
//!
//! Everything here requires the `3d` feature, which is enabled by default.

mod mesh;

pub use mesh::{Mesh, MeshBuilder, Vertex};