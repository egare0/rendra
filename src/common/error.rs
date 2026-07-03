use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendraError {
    #[error("Failed to request rendering adapter")]
    AdapterRequestFailed,
    #[error("Failed to request rendering device: {0}")]
    DeviceRequestFailed(String),
    #[error("Failed to create rendering surface: {0}")]
    SurfaceCreationFailed(String),
    #[error("Surface operation failed: {0}")]
    SurfaceError(String),
    #[error("Mesh has no vertices or no indices")]
    EmptyMesh,
    #[error("Texture data length {actual} does not match expected length {expected} (width * height * 4)")]
    InvalidTextureData { expected: usize, actual: usize },
}