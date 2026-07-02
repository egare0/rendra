use crate::{Device, RendraError};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

pub struct Surface {
    pub(crate) handle: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl Surface {
    pub fn new<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static>(device: &Device, window: Arc<W>, width: u32, height: u32, vsync: bool) -> Result<Self, RendraError> {
        let surface = device.instance.create_surface(window).map_err(|err| RendraError::SurfaceCreationFailed(err.to_string()))?;
        let surface_caps = surface.get_capabilities(&device.adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);

        let present_mode = if vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device.handle, &config);

        Ok(Self { handle: surface, config })
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.handle.configure(&device.handle, &self.config);
        }
    }

    /// Current swapchain width in pixels.
    #[inline]
    #[must_use]
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Current swapchain height in pixels.
    #[inline]
    #[must_use]
    pub fn height(&self) -> u32 {
        self.config.height
    }
}