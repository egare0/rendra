use crate::{Device, RendraError};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

/// A window's swapchain: the wgpu surface plus its current configuration.
///
/// Create one `Surface` per window and share a single `Device` across all
/// of them. Build one with [`Surface::builder`].
pub struct Surface {
    pub(crate) handle: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl Surface {
    /// Starts building a surface for `window`, configured at `width` x
    /// `height`.
    #[must_use]
    pub fn builder<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static>(window: Arc<W>, width: u32, height: u32) -> SurfaceBuilder<W> {
        SurfaceBuilder {
            window,
            width,
            height,
            vsync: true,
        }
    }

    /// Resizes the swapchain. Ignored if either dimension is zero, which
    /// happens transiently when a window is minimized.
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

/// Builds a [`Surface`] with optional settings.
pub struct SurfaceBuilder<W> {
    window: Arc<W>,
    width: u32,
    height: u32,
    vsync: bool,
}

impl<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static> SurfaceBuilder<W> {
    /// Enables or disables vertical sync. Defaults to enabled.
    #[inline]
    #[must_use]
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Creates the surface.
    pub fn build(self, device: &Device) -> Result<Surface, RendraError> {
        let surface = device.instance.create_surface(self.window).map_err(|err| RendraError::SurfaceCreationFailed(err.to_string()))?;
        let surface_caps = surface.get_capabilities(&device.adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);

        let present_mode = if self.vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: self.width.max(1),
            height: self.height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device.handle, &config);

        Ok(Surface { handle: surface, config })
    }
}