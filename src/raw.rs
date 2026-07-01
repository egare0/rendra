use crate::{Device, Frame, Surface};

pub trait RawDeviceAccess {
    fn raw_device(&self) -> &wgpu::Device;
    fn raw_queue(&self) -> &wgpu::Queue;
}

impl RawDeviceAccess for Device {
    #[inline]
    fn raw_device(&self) -> &wgpu::Device {
        &self.handle
    }

    #[inline]
    fn raw_queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

pub trait RawSurfaceAccess {
    fn raw_surface(&self) -> &wgpu::Surface<'static>;
    fn raw_config(&self) -> &wgpu::SurfaceConfiguration;
}

impl RawSurfaceAccess for Surface {
    #[inline]
    fn raw_surface(&self) -> &wgpu::Surface<'static> {
        &self.handle
    }

    #[inline]
    fn raw_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}

pub trait RawFrameAccess {
    fn raw_encoder(&mut self) -> &mut wgpu::CommandEncoder;
    fn raw_view(&self) -> &wgpu::TextureView;
}

impl RawFrameAccess for Frame {
    #[inline]
    fn raw_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }

    #[inline]
    fn raw_view(&self) -> &wgpu::TextureView {
        &self.view
    }
}