use crate::common::layouts::{frame_globals_layout, per_draw_layout, PER_DRAW_SIZE};
use crate::{Color, Device, RendraError, Surface};
use bytemuck::{Pod, Zeroable};
use std::cell::Cell;

/// Maximum number of draw calls per frame. Each draw claims one slot of
/// the shared per-draw uniform buffer; exceeding this returns an error
/// instead of silently overwriting earlier draws.
pub(crate) const DRAW_CAPACITY: u64 = 1024;

/// Raw light data as it lives in the frame-globals uniform buffer. This is
/// plain bytes to `common` - the light types that fill it meaningfully
/// live in `render3d`. Colors are premultiplied by intensity on the CPU.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub(crate) struct LightsRaw {
    /// rgb premultiplied by intensity, w unused.
    pub ambient: [f32; 4],
    /// xyz direction, w unused.
    pub dir_direction: [f32; 4],
    /// rgb premultiplied by intensity, w unused.
    pub dir_color: [f32; 4],
    /// xyz position, w = range.
    pub point_position: [f32; 4],
    /// rgb premultiplied by intensity, w unused.
    pub point_color: [f32; 4],
}

fn align_to(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

/// Owns per-frame draw orchestration: the frame-globals and per-draw
/// uniform buffers, and opening/submitting each frame's command encoder.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) lights: LightsRaw,
    lights_buffer: wgpu::Buffer,
    frame_globals_bind_group: wgpu::BindGroup,
    draw_buffer: wgpu::Buffer,
    draw_bind_group: wgpu::BindGroup,
    draw_slot_size: u64,
    draw_cursor: Cell<u64>,
}

impl Renderer {
    /// Creates a renderer bound to `device`.
    #[must_use]
    pub fn new(device: &Device) -> Self {
        let handle = &device.handle;

        let lights_buffer = handle.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rendra Lights Buffer"),
            size: std::mem::size_of::<LightsRaw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });
        let frame_globals_bind_group = handle.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rendra Frame Globals Bind Group"),
            layout: &frame_globals_layout(handle),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_buffer.as_entire_binding()
            }]
        });

        let alignment = handle.limits().min_uniform_buffer_offset_alignment as u64;
        let draw_slot_size = align_to(PER_DRAW_SIZE, alignment);
        let draw_buffer = handle.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rendra Per-Draw Buffer"),
            size: draw_slot_size * DRAW_CAPACITY,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });
        let draw_bind_group = handle.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rendra Per-Draw Bind Group"),
            layout: &per_draw_layout(handle),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &draw_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(PER_DRAW_SIZE)
                })
            }]
        });

        Self {
            device: handle.clone(),
            queue: device.queue.clone(),
            lights: LightsRaw::default(),
            lights_buffer,
            frame_globals_bind_group,
            draw_buffer,
            draw_bind_group,
            draw_slot_size,
            draw_cursor: Cell::new(0),
        }
    }

    /// Draws one frame into `surface`.
    ///
    /// If `clear_color` is `Some`, the color target (and the depth buffer,
    /// if the surface has one) is cleared before `draw` runs. Everything
    /// `draw` does lands in a single render pass.
    pub fn render(&mut self, surface: &mut Surface, clear_color: Option<Color>, draw: impl FnOnce(&mut Frame<'_>) -> Result<(), RendraError>) -> Result<(), RendraError> {
        self.draw_cursor.set(0);
        self.queue.write_buffer(&self.lights_buffer, 0, bytemuck::bytes_of(&self.lights));

        let surface_texture = match surface.handle.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                surface.handle.configure(&self.device, &surface.config);
                texture
            },
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
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Rendra Command Encoder")
        });

        let (load, depth_load) = match clear_color {
            Some(color) => (wgpu::LoadOp::Clear(color.into()), wgpu::LoadOp::Clear(1.0)),
            None => (wgpu::LoadOp::Load, wgpu::LoadOp::Load)
        };

        {
            let depth_stencil_attachment = depth_view.as_ref().map(|view| wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load,
                    store: wgpu::StoreOp::Store
                }),
                stencil_ops: None
            });

            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Rendra Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load,
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None
            });

            let mut frame = Frame {
                pass,
                queue: &self.queue,
                frame_globals_bind_group: &self.frame_globals_bind_group,
                draw_buffer: &self.draw_buffer,
                draw_bind_group: &self.draw_bind_group,
                draw_slot_size: self.draw_slot_size,
                draw_cursor: &self.draw_cursor,
                width: surface.width(),
                height: surface.height()
            };

            draw(&mut frame)?;
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(surface_texture);

        Ok(())
    }
}

/// One frame's worth of drawing state: an open render pass plus the shared
/// GPU resources draw calls record into.
///
/// `common` doesn't know about meshes or materials - `render3d` and
/// `render2d` add drawing methods to this type.
pub struct Frame<'a> {
    pub(crate) pass: wgpu::RenderPass<'a>,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) frame_globals_bind_group: &'a wgpu::BindGroup,
    pub(crate) draw_buffer: &'a wgpu::Buffer,
    pub(crate) draw_bind_group: &'a wgpu::BindGroup,
    pub(crate) draw_slot_size: u64,
    pub(crate) draw_cursor: &'a Cell<u64>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Frame<'_> {
    /// Width in pixels of the surface this frame draws into.
    #[inline]
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels of the surface this frame draws into.
    #[inline]
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
}