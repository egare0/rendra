use crate::common::layouts::{object_bind_group_layout, camera_bind_group_layout};
use crate::render3d::Vertex;
use crate::{Color, Device, RendraError, Surface, Texture};
use wgpu::util::DeviceExt;

const UNLIT_SHADER: &str = include_str!("unlit.wgsl");

/// A material using rendra's built-in unlit shader: an optionally
/// textured, tinted surface with no lighting calculations.
pub struct Material {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Material {
    /// Starts building an unlit material.
    #[inline]
    #[must_use]
    pub fn unlit() -> MaterialBuilder<'static> {
        MaterialBuilder {
            texture: None,
            tint: Color::WHITE
        }
    }
}

/// Builds an unlit [`Material`].
pub struct MaterialBuilder<'a> {
    texture: Option<&'a Texture>,
    tint: Color,
}

impl<'a> MaterialBuilder<'a> {
    /// Sets the albedo texture. If never called, falls back to a solid
    /// whit 1x1 texture - combined with `tint`, that gives a flat-colored
    /// material with no image needed.
    #[inline]
    #[must_use]
    pub fn texture(mut self, texture: &'a Texture) -> Self {
        self.texture = Some(texture);
        self
    }

    /// Sets the tint color, multiplied with the sampled texture color.
    /// Defaults to white (no tint).
    #[inline]
    #[must_use]
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    /// Builds the material, compiling a pipeline that targets `surface`'s
    /// color format.
    pub fn build(self, device: &Device, surface: &Surface) -> Result<Material, RendraError> {
        let fallback;
        let texture = match self.texture {
            Some(texture) => texture,
            None => {
                fallback = Texture::builder(1, 1, &[255, 255, 255, 255]).build(device)?;
                &fallback
            }
        };

        let handle = &device.handle;

        let tint_data: [f32; 4] = [
            self.tint.r,
            self.tint.g,
            self.tint.b,
            self.tint.a,
        ];
        let tint_buffer = handle.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rendra Material Tint Buffer"),
            contents: bytemuck::bytes_of(&tint_data),
            usage: wgpu::BufferUsages::UNIFORM
        });

        let material_layout = material_bind_group_layout(handle);
        let bind_group = handle.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rendra Unlit Material Bind Group"),
            layout: &material_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: tint_buffer.as_entire_binding(),
                },
            ],
        });

        let camera_layout = camera_bind_group_layout(handle);
        let object_layout = object_bind_group_layout(handle);

        let shader = handle.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rendra Unlit Shader"),
            source: wgpu::ShaderSource::Wgsl(UNLIT_SHADER.into()),
        });

        let pipeline_layout = handle.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rendra Unlit Pipeline Layout"),
            bind_group_layouts: &[Some(&camera_layout), Some(&material_layout), Some(&object_layout)],
            immediate_size: 0,
        });

        let pipeline = handle.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rendra Unlit Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::layout())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface.color_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Material {
            pipeline,
            bind_group,
        })
    }
}

/// Group 1: the unlit material's own bindings - albedo texture, its
/// sampler, and a tint uniform.
fn material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Rendra Unlit Material Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}