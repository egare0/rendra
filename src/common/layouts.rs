//! Shared bind group layouts. wgpu compares explicitly-created layouts
//! structurally, so Renderer and Material can each create their own copy
//! of these and the resulting pipelines and bind groups stay compatible.

/// Group 0: frame globals - light data, written once per frame.
pub(crate) fn frame_globals_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Rendra Frame Globals Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

/// Group 2: per-draw data - a model matrix and a view-projection matrix,
/// one 128-byte slice of a shared buffer per draw call, selected with a
/// dynamic offset.
pub(crate) fn per_draw_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Rendra Per-Draw Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: wgpu::BufferSize::new(PER_DRAW_SIZE),
            },
            count: None,
        }],
    })
}

/// Two mat4x4<f32>: model + view_proj.
pub(crate) const PER_DRAW_SIZE: u64 = 128;