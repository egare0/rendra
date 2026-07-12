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

    #[cfg(feature = "image")]
    #[error("Failed to read image file: {0}")]
    ImageReadFailed(String),

    #[cfg(feature = "image")]
    #[error("Failed to decode image: {0}")]
    ImageDecodeFailed(String),

    #[error("Too many draw calls with distinct transforms in one frame (max 1024)")]
    TooManyDraws,
    #[error("texture slot '{0}' is declared more than once")]
    DuplicateTextureSlot(String),
    #[error("shader has no texture slot named '{0}'")]
    UnknownTextureSlot(String),
}