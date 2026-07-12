use crate::{Device, RendraError};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

struct DepthAttachment {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

fn create_depth_attachment(device: &wgpu::Device, width: u32, height: u32) -> DepthAttachment {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Rendra Depth Texture"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[]
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    DepthAttachment { texture, view }
}

/// A window's swapchain: the wgpu surface, its configuration, and an
/// optional depth buffer.
///
/// Create one `Surface` per window and share a single `Device` across all
/// of them. Build one with [`Surface::builder`].
pub struct Surface {
    pub(crate) handle: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    depth: Option<DepthAttachment>,
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
            depth: false
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

    pub(crate) fn depth_view(&self) -> Option<&wgpu::TextureView> {
        self.depth.as_ref().map(|d| &d.view)
    }

    /// The swapchain's color format. `Material` needs this to build a
    /// pipeline that targets this surface.
    pub(crate) fn color_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Whether this surface was built with a depth buffer. Pipelines
    /// targeting this surface have to match.
    pub(crate) fn depth_enabled(&self) -> bool {
        self.depth.is_some()
    }
}

/// Builds a [`Surface`] with optional settings.
pub struct SurfaceBuilder<W> {
    window: Arc<W>,
    width: u32,
    height: u32,
    vsync: bool,
    depth: bool
}

impl<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static> SurfaceBuilder<W> {
    /// Enables or disables vertical sync. Defaults to enabled.
    #[inline]
    #[must_use]
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Enables or disables a depth buffer (`Depth32Float`). Defaults to
    /// disabled - turn it on for 3D rendering, leave it off for 2D.
    #[inline]
    #[must_use]
    pub fn depth(mut self, depth: bool) -> Self {
        self.depth = depth;
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

        let depth = self.depth.then(|| {
            create_depth_attachment(&device.handle, self.width.max(1), self.height.max(1))
        });

        Ok(Surface {
            handle: surface,
            config,
            depth
        })
    }
}