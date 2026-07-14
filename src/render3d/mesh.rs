use crate::{Device, RendraError};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use glam::Vec3;

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

    /// A cube centered at the origin with the given edge length. 24
    /// vertices - four per face, so every face gets a proper flat normal.
    #[must_use]
    pub fn cube(size: f32) -> MeshBuilder {
        let h = size * 0.5;
        let faces: [(Vec3, Vec3); 6] = [
            (Vec3::Z, Vec3::X),
            (Vec3::NEG_Z, Vec3::NEG_X),
            (Vec3::X, Vec3::NEG_Z),
            (Vec3::NEG_X, Vec3::Z),
            (Vec3::Y, Vec3::X),
            (Vec3::NEG_Y, Vec3::X)
        ];
        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);
        for (normal, tangent) in faces {
            let bitangent = normal.cross(tangent);
            let base = vertices.len() as u32;

            for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                let pos = normal * h + tangent * (u * 2.0 - 1.0) * h + bitangent * (1.0 - v * 2.0) * h;
                vertices.push(Vertex {
                    position: pos.to_array(),
                    normal: normal.to_array(),
                    tangent: [tangent.x, tangent.y, tangent.z, 1.0],
                    uv: [u, v]
                });
            }

            indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        }

        MeshBuilder { vertices, indices }
    }

    /// A flat square in the XZ plane, facing +Y, centered at the origin.
    #[must_use]
    pub fn plane(size: f32) -> MeshBuilder {
        let h = size * 0.5;
        let corners = [
            ([-h, 0.0, -h], [0.0, 0.0]),
            ([h, 0.0, -h], [1.0, 0.0]),
            ([h, 0.0, h], [1.0, 1.0]),
            ([-h, 0.0, h], [0.0, 1.0]),
        ];
        let vertices = corners.iter().map(|(pos, uv)| Vertex {
            position: *pos,
            normal: [0.0, 1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            uv: *uv
        }).collect();

        MeshBuilder { vertices, indices: vec![0, 2, 1, 0, 3, 2] }
    }

    /// A UV sphere with the given radius. `sectors` is the horizontal
    /// resolution, `stacks` the vertical - 32 and 16 look smooth at
    /// ordinary sizes.
    #[must_use]
    pub fn uv_sphere(radius: f32, sectors: u32, stacks: u32) -> MeshBuilder {
        let sectors = sectors.max(3);
        let stacks = stacks.max(2);
        let mut vertices = Vec::with_capacity(((stacks + 1) * (sectors + 1)) as usize);

        for stack in 0..=stacks {
            let phi = std::f32::consts::PI * stack as f32 / stacks as f32;
            let (sin_phi, cos_phi) = phi.sin_cos();

            for sector in 0..=sectors {
                let theta = std::f32::consts::TAU * sector as f32 / sectors as f32;
                let (sin_theta, cos_theta) = theta.sin_cos();
                let normal = Vec3::new(sin_phi * cos_theta, cos_phi, sin_phi * sin_theta);
                let tangent = Vec3::new(-sin_theta, 0.0, cos_theta);
                vertices.push(Vertex {
                    position: (normal * radius).to_array(),
                    normal: normal.to_array(),
                    tangent: [tangent.x, tangent.y, tangent.z, 1.0],
                    uv: [sector as f32 / sectors as f32, stack as f32 / stacks as f32]
                });
            }
        }

        let ring = sectors + 1;
        let mut indices = Vec::with_capacity((stacks * sectors * 6) as usize);
        
        for stack in 0..stacks {
            for sector in 0..sectors {
                let a = stack * ring + sector;
                let b = a + ring;
                indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
            }
        }

        MeshBuilder { vertices, indices }
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