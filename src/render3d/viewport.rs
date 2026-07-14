use glam::camera::rh::proj::directx;
use glam::{Mat3, Mat4, Quat, Vec3};

/// Projection Settings for a [`Viewport3D`].
#[derive(Debug, Clone, Copy)]
enum Projection {
    Perspective {
        fov_y: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        height: f32,
        near: f32,
        far: f32,
    }
}

/// A camera's worth of math: a projection, a position and an orientation.
///
/// This is not a camera entity - input, movement and scene logic belong to
/// whatever drives the render loop. A game engine typically keeps its own
/// camera object and copies its transform in here every frame.
///
/// Aspect ratio is never stored: `Frame::draw` derives it from the surface
/// being drawn into at draw time, so window resizes never touch this type.
#[derive(Debug, Clone, Copy)]
pub struct Viewport3D {
    projection: Projection,
    position: Vec3,
    rotation: Quat,
}

impl Viewport3D {
    /// Creates a perspective viewport. `fov_y` is the vertical field of
    /// view in radians.
    #[inline]
    #[must_use]
    pub fn perspective(fov_y: f32, near: f32, far: f32) -> Self {
        Self {
            projection: Projection::Perspective { fov_y, near, far },
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        }
    }

    /// Creates an orthographic viewport. `height` is the view volume's
    /// height in world units; width follows from the aspect ratio.
    #[inline]
    #[must_use]
    pub fn orthographic(height: f32, near: f32, far: f32) -> Self {
        Self {
            projection: Projection::Orthographic { height, near, far },
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        }
    }

    /// Moves the viewport to `position`.
    #[inline]
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Current position.
    #[inline]
    #[must_use]
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Sets the orientation directly.
    #[inline]
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    /// Current orientation.
    #[inline]
    #[must_use]
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    /// Points the viewport at `target` from its current position, keeping
    /// +Y as up. Looking straight up or down falls back to the last valid
    /// right vector direction.
    pub fn look_at(&mut self, target: Vec3) {
        let forward = (target - self.position).normalize_or(Vec3::NEG_Z);
        let right = forward.cross(Vec3::Y).normalize_or(Vec3::X);
        let up = right.cross(forward);
        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, -forward));
    }

    /// The combined view-projection matrix at the given aspect ratio.
    ///
    /// You rarely call this yourself - `Frame::draw` does, with the aspect
    /// of the surface being drawn into.
    #[must_use]
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let view = Mat4::from_rotation_translation(self.rotation, self.position).inverse();
        let projection = match self.projection {
            Projection::Perspective { fov_y, near, far } => directx::perspective(fov_y, aspect, near, far),
            Projection::Orthographic { height, near, far } => {
                let half_h = height * 0.5;
                let half_w = half_h * aspect;
                directx::orthographic(-half_w, half_w, -half_h, half_h, near, far)
            }
        };
        projection * view
    }
}