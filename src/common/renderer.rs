use crate::{Color, Device, RendraError, Surface};

/// Owns per-frame draw orchestration: acquiring the next frame, building a
/// command encoder, and presenting when you're done.
///
/// A `Renderer` is bound to a `Device` at construction but takes a
/// `Surface` on every call to `render`, so one `Renderer` can draw into as
/// many windows as you have.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl Renderer {
    /// Creates a renderer bound to `device`.
    ///
    /// This clones the device's inner wgpu handles - wgpu reference-counts
    /// them internally, so this is cheap and safe to call more than once.
    #[must_use]
    pub fn new(device: &Device) -> Self {
        Self {
            device: device.handle.clone(),
            queue: device.queue.clone(),
        }
    }

    /// Draws one frame into `surface`.
    ///
    /// Acquires the next swapchain texture, runs `draw` against a fresh
    /// [`Frame`], then submits and presents. Occluded or timed-out frames
    /// are silently skipped, which happens naturally when a window is
    /// minimized or being resized.
    pub fn render(&mut self, surface: &mut Surface, draw: impl FnOnce(&mut Frame)) -> Result<(), RendraError> {
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
        let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Rendra Command Encoder"),
        });
        let mut frame = Frame { view, encoder };

        draw(&mut frame);

        self.queue.submit(std::iter::once(frame.encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

/// One frame's worth of drawing state: a target view and a command encoder.
pub struct Frame {
    pub(crate) view: wgpu::TextureView,
    pub(crate) encoder: wgpu::CommandEncoder,
}

impl Frame {
    /// Clears the frame to a solid color.
    pub fn clear(&mut self, color: Color) {
        let _pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color.into()),
                    store: wgpu::StoreOp::Store
                }
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None
        });
    }
}