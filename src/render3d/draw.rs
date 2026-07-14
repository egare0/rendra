use crate::{
    common::DRAW_CAPACITY,
    render3d::{
        Material,
        Mesh,
        Viewport3D
    },
    Frame,
    RendraError
};
use glam::Mat4;

impl Frame<'_> {
    /// Draws `mesh` with `material`, placed in the world by `transform`,
    /// seen through `viewport`.
    ///
    /// Only available with the `3d` feature. Returns
    /// [`RendraError::TooManyDraws`] past 1024 draw calls in one frame.
    pub fn draw(&mut self, mesh: &Mesh, material: &Material, viewport: &Viewport3D, transform: Mat4) -> Result<(), RendraError> {
        let offset = self.draw_cursor.get();

        if offset >= self.draw_slot_size * DRAW_CAPACITY {
            return Err(RendraError::TooManyDraws)
        }

        self.draw_cursor.set(offset + self.draw_slot_size);

        let aspect = self.width as f32 / self.height as f32;
        let data: [Mat4; 2] = [transform, viewport.view_projection(aspect)];

        self.queue.write_buffer(self.draw_buffer, offset, bytemuck::cast_slice(&data));
        self.pass.set_pipeline(&material.pipeline);
        self.pass.set_bind_group(0, self.frame_globals_bind_group, &[]);
        self.pass.set_bind_group(1, &material.bind_group, &[]);
        self.pass.set_bind_group(2, self.draw_bind_group, &[offset as u32]);
        self.pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.pass.draw_indexed(0..mesh.index_count, 0, 0..1);

        Ok(())
    }
}