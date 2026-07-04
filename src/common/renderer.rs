use crate::{Color, Device, RendraError, Surface, common::layouts::{camera_bind_group_layout, object_bind_group_layout}};
use std::cell::Cell;

/// Maximum number of draw calls with distinct transforms per frame. Each
/// draw claims one slot of the shared object uniform buffer via a dynamic
/// offset; exceeding this returns an error instead of silently corrupting
/// earlier draws in the same frame.
pub(crate) const OBJECT_CAPACITY: u64 = 1024;

fn align_to(value: wgpu::BufferAddress, alignment: wgpu::BufferAddress) -> wgpu::BufferAddress {
    value.div_ceil(alignment) * alignment
}

/// Owns per-frame draw orchestration: the camera and per-object uniform
/// buffers, and opening/submitting each frame's command encoder.
///
/// A `Renderer` is bound to a `Device` at construction but takes a
/// `Surface` on every call to `render`, so one `Renderer` can draw into as
/// many windows as you have.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    object_buffer: wgpu::Buffer,
    object_bind_group: wgpu::BindGroup,
    object_slot_size: wgpu::BufferAddress,
    object_cursor: Cell<wgpu::BufferAddress>,
}

impl Renderer {
    /// Creates a renderer bound to `device`.
    ///
    /// This clones the device's inner wgpu handles - wgpu reference-counts
    /// them internally, so this is cheap and safe to call more than once.
    /// Also allocates the camera and per-object uniform buffers used by
    /// every draw call.
    #[must_use]
    pub fn new(device: &Device) -> Self {
        let handle = &device.handle;

        let camera_layout = camera_bind_group_layout(handle);
        let camera_buffer = handle.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rendra Camera Buffer"),
            size: size_of::<glam::Mat4>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = handle.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rendra Camera Bind Group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }]
        });

        let alignment = handle.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let object_slot_size = align_to(size_of::<glam::Mat4>() as wgpu::BufferAddress, alignment);

        let object_layout = object_bind_group_layout(handle);
        let object_buffer = handle.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rendra Object Buffer"),
            size: object_slot_size * OBJECT_CAPACITY,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let object_bind_group = handle.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rendra Object Bind Group"),
            layout: &object_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &object_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<glam::Mat4>() as u64),
                }),
            }],
        });

        Self {
            device: handle.clone(),
            queue: device.queue.clone(),
            camera_buffer,
            camera_bind_group,
            object_buffer,
            object_bind_group,
            object_slot_size,
            object_cursor: Cell::new(0),
        }
    }

    /// Draws one frame into `surface`.
    ///
    /// If `clear_color` is `Some`, the color target (and the depth buffer,
    /// if the surface has one) is cleared before `draw` runs. Pass `None`
    /// to draw over whatever the surface already holds.
    ///
    /// Opens a single render pass covering everything `draw` does against
    /// the resulting [`Frame`], then submits and presents. Occluded or
    /// timed-out frames are silently skipped, which happens naturally when
    /// a window is minimized or being resized.
    pub fn render(&mut self, surface: &mut Surface, clear_color: Option<Color>, draw: impl FnOnce(&mut Frame<'_>) -> Result<(), RendraError>) -> Result<(), RendraError> {
        self.object_cursor.set(0);

        let surface_texture = match surface.handle.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                surface.handle.configure(&self.device, &surface.config);
                texture
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                surface.handle.configure(&self.device, &surface.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RendraError::SurfaceError("Validation error".to_string()));
            }
        };

        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = surface.depth_view().cloned();
        let mut encoder =self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Rendra Command Encoder"),
        });

        let (load, depth_load) = match clear_color {
            Some(color) => (wgpu::LoadOp::Clear(color.into()), wgpu::LoadOp::Clear(1.0)),
            None => (wgpu::LoadOp::Load, wgpu::LoadOp::Load),
        };

        {
            let depth_stencil_attachment = depth_view.as_ref().map(|view| wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load,
                    store: wgpu::StoreOp::Store
                }),
                stencil_ops: None,
            });

            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Rendra Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let mut frame = Frame {
                pass,
                queue: &self.queue,
                camera_buffer: &self.camera_buffer,
                camera_bind_group: &self.camera_bind_group,
                object_buffer: &self.object_buffer,
                object_bind_group: &self.object_bind_group,
                object_slot_size: self.object_slot_size,
                object_cursor: &self.object_cursor,
            };

            draw(&mut frame)?;
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(surface_texture);

        Ok(())
    }
}

/// One frame's worth of drawing state: an open render pass, plus the
/// shared camera and per-object GPU resources needed to issue draw calls
/// into it.
///
/// `common` doesn't know about meshes or materials - drawing them is added
/// by `render3d`'s `Draw3D` trait, implemented for this type.
pub struct Frame<'a> {
    pub(crate) pass: wgpu::RenderPass<'a>,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) camera_buffer: &'a wgpu::Buffer,
    pub(crate) camera_bind_group: &'a wgpu::BindGroup,
    pub(crate) object_buffer: &'a wgpu::Buffer,
    pub(crate) object_bind_group: &'a wgpu::BindGroup,
    pub(crate) object_slot_size: wgpu::BufferAddress,
    pub(crate) object_cursor: &'a Cell<wgpu::BufferAddress>,
}