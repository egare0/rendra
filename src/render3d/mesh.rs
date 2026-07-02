use crate::{Device, RendraError};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// A single point in a mesh: position, normal, tangent and UV, ready for a
/// PBR-style vertex shader.
///
/// `tangent` is a 4-component vector - xyz is the tangent direction, w
/// holds the bitangent's handedness (+1.0 or -1.0), which is the usual way
/// to avoid storing a full bitangent per vertex.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub uv: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x4,
        3 => Float32x2,
    ];

    /// Describes this struct's memory layout to wgpu, for use when building
    /// a render pipeline.
    #[must_use]
    pub const fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// GPU-resident geometry: a vertex buffer and an index buffer
pub struct Mesh {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
}

impl Mesh {
    /// Starts building a mesh.
    #[inline]
    #[must_use]
    pub fn builder() -> MeshBuilder {
        MeshBuilder::default()
    }
}

/// Builds a [`Mesh`] from vertex and index data.
#[derive(Default)]
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    /// Sets the mesh's vertices.
    #[inline]
    #[must_use]
    pub fn vertices(mut self, vertices: Vec<Vertex>) -> Self {
        self.vertices = vertices;
        self
    }

    /// Sets the mesh's indices.
    #[inline]
    #[must_use]
    pub fn indices(mut self, indices: Vec<u32>) -> Self {
        self.indices = indices;
        self
    }

    /// Uploads the vertex and index data to the GPU.
    ///
    /// Fails if either list is empty.
    pub fn build(self, device: &Device) -> Result<Mesh, RendraError> {
        if self.vertices.is_empty() || self.indices.is_empty() {
            return Err(RendraError::EmptyMesh);
        }

        let vertex_buffer = device.handle.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rendra Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.handle.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rendra Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(Mesh {
            vertex_buffer,
            index_buffer,
            index_count: self.indices.len() as u32,
        })
    }
}