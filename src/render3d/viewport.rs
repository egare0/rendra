use glam::Mat4;
use glam::camera::rh::proj::directx;

/// Produces a projection matrix and combines it with a view matrix you
/// supply.
///
/// rendra has no camera entity of its own - position, rotation and
/// movement all belong to whatever drives the render loop (a game engine,
/// an editor, a hand-rolled fly cam, anything). `Viewport3D` only knows how
/// to turn a view matrix into a combined view-projection matrix.
///
/// Uses glam's `directx`-convention projection constructors, which match
/// the depth range (0-1) and NDC that wgpu/WebGPU expects regardless of
/// which backend (Vulkan, Metal, DX12) is actually running underneath.
#[derive(Debug, Clone, Copy)]
pub struct Viewport3D {
    projection: Mat4,
}

impl Viewport3D {
    /// Creates a perspective projection.
    ///
    /// `fov_y_radians` is the vertical field of view. `aspect` is the
    /// viewport's width divided by its height - use
    /// `surface.width() as f32 / surface.height() as f32`.
    #[inline]
    #[must_use]
    pub fn perspective(fov_y_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            projection: directx::perspective(fov_y_radians, aspect, near, far),
        }
    }

    /// Creates an orthographic projection from the given clipping planes.
    #[inline]
    #[must_use]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            projection: directx::orthographic(left, right, bottom, top, near, far),
        }
    }

    /// Combines this viewport's projection with `view` into a single
    /// view-projection matrix.
    #[inline]
    #[must_use]
    pub fn view_projection(&self, view: Mat4) -> Mat4 {
        self.projection * view
    }
}