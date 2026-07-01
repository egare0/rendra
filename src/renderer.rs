use crate::{Color, RendraError};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

pub struct Renderer {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl Renderer {
    #[inline]
    #[must_use]
    pub fn builder() -> RendererBuilder {
        RendererBuilder::default()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, render_pass: impl FnOnce(&mut Frame)) -> Result<(), RendraError> {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                self.surface.configure(&self.device, &self.config);
                texture
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
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

        render_pass(&mut frame);

        self.queue.submit(std::iter::once(frame.encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

pub struct Frame {
    pub(crate) view: wgpu::TextureView,
    pub(crate) encoder: wgpu::CommandEncoder,
}

impl Frame {
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
            multiview_mask: None,
        });
    }
}

#[derive(Default)]
pub struct RendererBuilder {
    vsync: bool,
}

impl RendererBuilder {
    #[must_use]
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }
    
    pub fn build<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static>(self, window: Arc<W>, width: u32, height: u32, ) -> Result<Renderer, RendraError> {
        pollster::block_on(self.build_async(window, width, height))
    }

    async fn build_async<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static>(self, window: Arc<W>, width: u32, height: u32, ) -> Result<Renderer, RendraError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None
        });

        let surface = instance.create_surface(window).map_err(|e| RendraError::SurfaceCreationFailed(e.to_string()))?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.map_err(|_| RendraError::AdapterRequestFailed)?;


        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Rendra Device"),
            ..Default::default()
        }).await.map_err(|e| RendraError::DeviceRequestFailed(e.to_string()))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);

        let present_mode = if self.vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device, &config);

        Ok(Renderer {
            surface,
            device,
            queue,
            config
        })
    }
}