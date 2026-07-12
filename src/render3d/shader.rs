use crate::{Device, RendraError};

/// A compiled shader plus the metadata Material needs to bind resources
/// to it: which texture slots exist and whether lighting is enabled.
///
/// Build one either by composing options with [`Shader::builder`] - rendra
/// generates the WGSL - or by bringing your own source with
/// [`Shader::from_wgsl`].
pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) slots: Vec<String>,
    pub(crate) lit: bool
}

impl Shader {
    /// Starts composing a shader. rendra writes the WGSL.
    #[inline]
    #[must_use]
    pub fn builder() -> ShaderBuilder {
        ShaderBuilder {
            slots: Vec::new(),
            lit: false,
            custom_source: None,
        }
    }

    /// Starts from your own WGSL source instead of generated code.
    ///
    /// The source must follow rendra's bind group convention: group 0 is
    /// frame globals (lights uniform at binding 0), group 1 is the
    /// material (slot `i` puts its texture at binding `2i` and sampler at
    /// `2i + 1`, tint uniform after all slots), group 2 is per-draw data
    /// (`model` and `view_proj` matrices at binding 0). Entry points must
    /// be named `vs_main` and `fs_main`. Declare the slots you use with
    /// [`texture_slot`](ShaderBuilder::texture_slot) so Material can bind
    /// them.
    #[inline]
    #[must_use]
    pub fn from_wgsl(source: &str) -> ShaderBuilder {
        ShaderBuilder {
            slots: Vec::new(),
            lit: false,
            custom_source: Some(source.to_string()),
        }
    }
}

/// Composes a [`Shader`].
pub struct ShaderBuilder {
    slots: Vec<String>,
    lit: bool,
    custom_source: Option<String>,
}

impl ShaderBuilder {
    /// Declares a named texture slot. In generated shaders the slot is
    /// sampled with the mesh's UVs and multiplied into the output color;
    /// in `from_wgsl` shaders this only registers the slot for binding.
    #[must_use]
    pub fn texture_slot(mut self, name: &str) -> Self {
        self.slots.push(name.to_string());
        self
    }

    /// Enables lighting: ambient, one directional and one point light,
    /// Lambert diffuse. Has no effect on `from_wgsl` sources - custom
    /// shaders read the lights uniform themselves if they want it.
    #[inline]
    #[must_use]
    pub fn lit(mut self) -> Self {
        self.lit = true;
        self
    }

    /// Compiles the shader module.
    pub fn build(self, device: &Device) -> Result<Shader, RendraError> {
        for (i, name) in self.slots.iter().enumerate() {
            if self.slots[..i].contains(name) {
                return Err(RendraError::DuplicateTextureSlot(name.clone()));
            }
        }

        let source = match &self.custom_source {
            Some(src) => src.clone(),
            None => generate_wgsl(&self.slots, self.lit)
        };

        let module = device.handle.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rendra Shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        Ok(Shader {
            module,
            slots: self.slots,
            lit: self.lit
        })
    }
}

/// Writes the WGSL for a composed shader. Kept as one linear function on
/// purpose - reading it top to bottom shows exactly what any combination
/// of options produces.
fn generate_wgsl(slots: &[String], lit: bool) -> String {
    let mut s = String::with_capacity(2048);

    // Group 2: per-draw data.
    s.push_str(
        "struct DrawData {\n\
         \x20   model: mat4x4<f32>,\n\
         \x20   view_proj: mat4x4<f32>,\n\
         };\n\
         @group(2) @binding(0) var<uniform> draw_data: DrawData;\n\n",
    );

    // Group 0: frame globals, only declared when the shader uses them.
    if lit {
        s.push_str(
            "struct Lights {\n\
             \x20   ambient: vec4<f32>,\n\
             \x20   dir_direction: vec4<f32>,\n\
             \x20   dir_color: vec4<f32>,\n\
             \x20   point_position: vec4<f32>,\n\
             \x20   point_color: vec4<f32>,\n\
             };\n\
             @group(0) @binding(0) var<uniform> lights: Lights;\n\n",
        );
    }

    // Group 1: material - one texture + sampler pair per slot, tint last.
    for (i, name) in slots.iter().enumerate() {
        s.push_str(&format!(
            "@group(1) @binding({}) var t_{name}: texture_2d<f32>;\n\
             @group(1) @binding({}) var s_{name}: sampler;\n",
            i * 2,
            i * 2 + 1,
        ));
    }
    s.push_str(&format!(
        "struct MaterialData {{\n\
         \x20   tint: vec4<f32>,\n\
         }};\n\
         @group(1) @binding({}) var<uniform> material_data: MaterialData;\n\n",
        slots.len() * 2,
    ));

    // Vertex IO.
    s.push_str(
        "struct VsIn {\n\
         \x20   @location(0) position: vec3<f32>,\n\
         \x20   @location(1) normal: vec3<f32>,\n\
         \x20   @location(2) tangent: vec4<f32>,\n\
         \x20   @location(3) uv: vec2<f32>,\n\
         };\n\n\
         struct VsOut {\n\
         \x20   @builtin(position) clip_position: vec4<f32>,\n\
         \x20   @location(0) uv: vec2<f32>,\n",
    );
    if lit {
        s.push_str(
            "\x20   @location(1) world_normal: vec3<f32>,\n\
             \x20   @location(2) world_position: vec3<f32>,\n",
        );
    }
    s.push_str("};\n\n");

    // Vertex stage.
    s.push_str(
        "@vertex\n\
         fn vs_main(in: VsIn) -> VsOut {\n\
         \x20   var out: VsOut;\n\
         \x20   let world_position = draw_data.model * vec4<f32>(in.position, 1.0);\n\
         \x20   out.clip_position = draw_data.view_proj * world_position;\n\
         \x20   out.uv = in.uv;\n",
    );
    if lit {
        // Model matrix on the normal assumes uniform scale; the inverse
        // transpose for non-uniform scale is a roadmap item.
        s.push_str(
            "\x20   out.world_normal = (draw_data.model * vec4<f32>(in.normal, 0.0)).xyz;\n\
             \x20   out.world_position = world_position.xyz;\n",
        );
    }
    s.push_str("\x20   return out;\n}\n\n");

    // Fragment stage.
    s.push_str(
        "@fragment\n\
         fn fs_main(in: VsOut) -> @location(0) vec4<f32> {\n\
         \x20   var color = material_data.tint;\n",
    );
    for name in slots {
        s.push_str(&format!(
            "\x20   color = color * textureSample(t_{name}, s_{name}, in.uv);\n",
        ));
    }
    if lit {
        s.push_str(
            "\x20   let n = normalize(in.world_normal);\n\
             \x20   var light = lights.ambient.rgb;\n\
             \x20   light += max(dot(n, -normalize(lights.dir_direction.xyz)), 0.0) * lights.dir_color.rgb;\n\
             \x20   let to_point = lights.point_position.xyz - in.world_position;\n\
             \x20   let point_dist = length(to_point);\n\
             \x20   let point_range = max(lights.point_position.w, 0.0001);\n\
             \x20   let atten = clamp(1.0 - point_dist / point_range, 0.0, 1.0);\n\
             \x20   light += max(dot(n, to_point / point_dist), 0.0) * lights.point_color.rgb * atten * atten;\n\
             \x20   color = vec4<f32>(color.rgb * light, color.a);\n",
        );
    }
    s.push_str("\x20   return color;\n}\n");

    s
}