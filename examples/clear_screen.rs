use rendra::{Color, Device, Renderer, Surface};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, ControlFlow},
    window::{Window, WindowId}
};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    device: Option<Device>,
    surface: Option<Surface>,
    renderer: Option<Renderer>
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_title("Rendra - Clear Screen Test")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let device = Device::new().expect("Failed to initialize Device");
        let surface = Surface::builder(window.clone(), 800, 600)
            .vsync(true)
            .depth(true)
            .build(&device)
            .expect("Failed to create Surface");
        let renderer = Renderer::new(&device);

        self.window = Some(window);
        self.device = Some(device);
        self.surface = Some(surface);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let (Some(device), Some(surface)) = (self.device.as_ref(), self.surface.as_mut()) {
                    surface.resize(device, size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(surface)) = (self.renderer.as_mut(), self.surface.as_mut()) {
                    renderer.render(surface, |frame| {
                            frame.clear(Color::BLACK);
                        }).unwrap();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}