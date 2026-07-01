use crate::{Frame, Renderer};

pub trait RawAccess {
    fn raw_device(&self) -> &wgpu::Device;
    fn raw_queue(&self) -> &wgpu::Queue;
    fn raw_surface(&self) -> &wgpu::Surface<'static>;
    fn raw_config(&self) -> &wgpu::SurfaceConfiguration;
}

impl RawAccess for Renderer {
    #[inline]
    fn raw_device(&self) -> &wgpu::Device {
        &self.device
    }

    #[inline]
    fn raw_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    #[inline]
    fn raw_surface(&self) -> &wgpu::Surface<'static> {
        &self.surface
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