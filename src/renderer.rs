use crate::RendraError;
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