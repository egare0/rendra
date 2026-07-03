use crate::{Device, RendraError};
use wgpu::util::DeviceExt;

/// How a texture is sampled between texel centers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Filter {
    /// Smooth interpolation between texels. The right choice for most
    /// photographic or painted textures.
    #[default]
    Linear,
    /// No interpolation - blocky, crisp texels. The right choice for
    /// pixel art.
    Nearest,
}

impl Filter {
    fn to_wgpu(self) -> wgpu::FilterMode {
        match self {
            Filter::Linear => wgpu::FilterMode::Linear,
            Filter::Nearest => wgpu::FilterMode::Nearest,
        }
    }
}

/// A GPU-resident 2D texture with its view and sampler.
pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
}

impl Texture {
    /// Starts building a texture from raw RGBA8 pixel data.
    ///
    /// `pixels` must contain exactly `width * height * 4` bytes - decode
    /// your PNG/JPEG/whatever before calling this, rendra doesn't do image
    /// decoding.
    #[inline]
    #[must_use]
    pub fn builder(width: u32, height: u32, pixels: &[u8]) -> TextureBuilder<'_> {
        TextureBuilder {
            width,
            height,
            pixels,
            filter: Filter::default(),
            srgb: true
        }
    }

    /// Loads and decodes an image file from disk, then uploads it as a
    /// texture with default settings (linear filtering, sRGB). Requires
    /// the `image` feature.
    ///
    /// For control over filtering or color space, decode yourself and use
    /// [`Texture::builder`] instead.
    #[cfg(feature = "image")]
    pub fn from_file<P: AsRef<std::path::Path>>(device: &Device, path: P) -> Result<Self, RendraError> {
        let bytes = std::fs::read(path).map_err(|err| RendraError::ImageReadFailed(err.to_string()))?;
        Self::from_bytes(device, &bytes)
    }

    /// Decodes an already-loaded image (PNG or JPEG) and uploads it as a
    /// texture with default settings. Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn from_bytes(device: &Device, bytes: &[u8]) -> Result<Self, RendraError> {
        use image::GenericImageView;

        let decoded = image::load_from_memory(bytes).map_err(|err| RendraError::ImageDecodeFailed(err.to_string()))?;
        let rgba = decoded.to_rgba8();
        let (width, height) = rgba.dimensions();

        Texture::builder(width, height, &rgba).build(device)
    }
}

/// Builds [`Texture`] with optional filtering and color-space settings.
pub struct TextureBuilder<'a> {
    width: u32,
    height: u32,
    pixels: &'a [u8],
    filter: Filter,
    srgb: bool,
}

impl<'a> TextureBuilder<'a> {
    /// Sets the sampling filter. Defaults to [`Filter::Linear`].
    #[inline]
    #[must_use]
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    /// Whether this texture holds sRGB color data. Defaults to `true` -
    /// turn it off for normal maps, roughness maps, and other data
    /// textures that aren't meant to be gamma-decoded.
    #[inline]
    #[must_use]
    pub fn srgb(mut self, srgb: bool) -> Self {
        self.srgb = srgb;
        self
    }

    /// Uploads the pixel data to the GPU.
    pub fn build(self, device: &Device) -> Result<Texture, RendraError> {
        let expected = self.width as usize * self.height as usize * 4;

        if self.pixels.len() != expected {
            return Err(RendraError::InvalidTextureData {
                expected,
                actual: self.pixels.len(),
            });
        }

        let format = if self.srgb {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        let texture = device.handle.create_texture_with_data(&device.queue, &wgpu::TextureDescriptor {
            label: Some("Rendra Texture"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[]

        }, wgpu::util::TextureDataOrder::LayerMajor, self.pixels);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let filter_mode = self.filter.to_wgpu();
        let sampler = device.handle.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Rendra Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter: match filter_mode {
                wgpu::FilterMode::Linear => wgpu::MipmapFilterMode::Linear,
                wgpu::FilterMode::Nearest => wgpu::MipmapFilterMode::Nearest,
            },
            ..Default::default()
        });

        Ok(Texture {
            texture,
            view,
            sampler
        })
    }
}