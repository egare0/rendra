use crate::RendraError;

/// Owns the GPU connection: the wgpu instance, the selected adapter, the
/// logical device and its command queue.
///
/// A `Device` is independent of any window. Create one per application and
/// share it across as many surfaces as you need — call `Surface::new` once
/// you have a window to draw into.
pub struct Device {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) handle: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl Device {
    /// Initializes the GPU connection, blocking the current thread until
    /// it's ready.
    ///
    /// This is the entry point for most applications. If you're already
    /// inside an async runtime and want to await initialization instead
    /// of blocking a thread, use [`Device::new_async`] instead.
    pub fn new() -> Result<Self, RendraError> {
        pollster::block_on(Self::new_async())
    }

    /// Initializes the GPU connection asynchronously.
    ///
    /// Picks a high-performance adapter and requests a device with default
    /// limits and no extra features enabled.
    pub async fn new_async() -> Result<Self, RendraError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }).await.map_err(|_| RendraError::AdapterRequestFailed)?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Rendra Device"),
            ..Default::default()
        }).await.map_err(|err| RendraError::DeviceRequestFailed(err.to_string()))?;

        Ok(Self {
            instance,
            adapter,
            handle: device,
            queue
        })
    }
}