use crate::common::{frame_globals_layout, per_draw_layout};
use crate::render3d::{Shader, Vertex};
use crate::{Color, Device, RendraError, Surface, Texture};
use wgpu::util::DeviceExt;

/// A shader bound to its resources: textures filling the shader's slots
/// and a tint color, with the render pipeline built and ready.
pub struct Material {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Material {
    /// Starts building a material for `shader`.
    #[inline]
    #[must_use]
    pub fn builder(shader: &Shader) -> MaterialBuilder<'_> {
        MaterialBuilder {
            shader,
            textures: Vec::new(),
            tint: Color::WHITE,
        }
    }
}

/// Builds a [`Material`].
pub struct MaterialBuilder<'a> {
    shader: &'a Shader,
    textures: Vec<(&'a str, &'a Texture)>,
    tint: Color,
}

impl<'a> MaterialBuilder<'a> {
    /// Assigns `texture` to the shader's slot named `name`. Slots left
    /// unassigned fall back to a 1x1 white texture, so tint alone gives a
    /// flat-colored material.
    #[inline]
    #[must_use]
    pub fn texture(mut self, name: &'a str, texture: &'a Texture) -> Self {
        self.textures.push((name, texture));
        self
    }

    /// Sets the tint color, multiplied into the output. Defaults to white.
    #[inline]
    #[must_use]
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    /// Builds the material, compiling a pipeline that matches `surface`'s
    /// color format and depth setting.
    pub fn build(self, device: &Device, surface: &Surface) -> Result<Material, RendraError> {
        for (name, _) in &self.textures {
            if !self.shader.slots.iter().any(|s| s == name) {
                return Err(RendraError::UnknownTextureSlot((*name).to_string()));
            }
        }

        let handle = &device.handle;

        // Fill every slot: assigned texture or the white fallback.
        let fallback = if self.textures.len() < self.shader.slots.len() {
            Some(Texture::builder(1, 1, &[255, 255, 255, 255]).build(device)?)
        } else {
            None
        };

        let resolved: Vec<&Texture> = self.shader.slots.iter().map(|slot| self.textures.iter()
            .find(|(name, _)| name == slot)
            .map(|(_, tex)| *tex)
            .unwrap_or_else(|| fallback.as_ref().unwrap()))
            .collect();

        let tint_color: wgpu::Color = self.tint.into();
        let tint_data: [f32; 4] = [
            tint_color.r as f32,
            tint_color.g as f32,
            tint_color.b as f32,
            tint_color.a as f32
        ];
        let tint_buffer = handle.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rendra Material Tint Buffer"),
            contents: bytemuck::bytes_of(&tint_data),
            usage: wgpu::BufferUsages::UNIFORM
        });

        // Group 1 layout: texture at 2i, sampler at 2i+1, tint last.
        let mut layout_entries = Vec::with_capacity(self.shader.slots.len() * 2 + 1);
        for i in 0..self.shader.slots.len() as u32 {
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i * 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float {
                        filterable: true
                    },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false
                },
                count: None,
            });

            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i * 2 + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }

        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.shader.slots.len() as u32 * 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        let material_layout = handle.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rendra Material Layout"),
            entries: &layout_entries,
        });

        let mut bind_entries = Vec::with_capacity(resolved.len() * 2 + 1);
        for (i, texture) in resolved.iter().enumerate() {
            bind_entries.push(wgpu::BindGroupEntry {
                binding: i as u32 * 2,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            bind_entries.push(wgpu::BindGroupEntry {
                binding: i as u32 * 2 + 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        bind_entries.push(wgpu::BindGroupEntry {
            binding: resolved.len() as u32 * 2,
            resource: tint_buffer.as_entire_binding(),
        });

        let bind_group = handle.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rendra Material Bind Group"),
            layout: &material_layout,
            entries: &bind_entries,
        });

        let pipeline_layout = handle.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rendra Material Pipeline Layout"),
            bind_group_layouts: &[
                Some(&frame_globals_layout(handle)),
                Some(&material_layout),
                Some(&per_draw_layout(handle)),
            ],
            immediate_size: 0,
        });

        let depth_stencil = surface.depth_enabled().then(|| wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let pipeline = handle.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rendra Material Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.shader.module,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::layout())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader.module,
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
            depth_stencil,
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