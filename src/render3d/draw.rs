use crate::common::OBJECT_CAPACITY;
use crate::render3d::{Material, Mesh};
use crate::{Frame, RendraError};
use glam::Mat4;

/// Adds mesh drawing to [`Frame`]. Import this trait to call
/// [`draw`](Draw3D::draw) - only available with the `3d` feature.
pub trait Draw3D {
    /// Draws `mesh` with `material`, positioned by `transform`, seen
    /// through `view_projection`.
    ///
    /// Returns [`RendraError::TooManyDraws`] if this frame has already
    /// drawn 1024 objects with distinct transforms.
    fn draw(&mut self, mesh: &Mesh, material: &Material, transform: Mat4, view_projection: Mat4, ) -> Result<(), RendraError>;
}

impl Draw3D for Frame<'_> {
    fn draw(&mut self, mesh: &Mesh, material: &Material, transform: Mat4, view_projection: Mat4) -> Result<(), RendraError> {
        let offset = self.object_cursor.get();
        let capacity = self.object_slot_size * OBJECT_CAPACITY;

        if offset >= capacity {
            return Err(RendraError::TooManyDraws);
        }

        self.object_cursor.set(offset + self.object_slot_size);

        self.queue.write_buffer(self.camera_buffer, 0, bytemuck::bytes_of(&view_projection));
        self.queue.write_buffer(self.object_buffer, offset, bytemuck::bytes_of(&transform));

        self.pass.set_pipeline(&material.pipeline);
        self.pass.set_bind_group(0, self.camera_bind_group, &[]);
        self.pass.set_bind_group(1, &material.bind_group, &[]);
        self.pass.set_bind_group(2, self.object_bind_group, &[offset as u32]);
        self.pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.pass.draw_indexed(0..mesh.index_count, 0, 0..1);

        Ok(())
    }
}