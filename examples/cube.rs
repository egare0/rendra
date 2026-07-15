use rendra::{
    glam::{Mat4, Vec3},
    render3d::{Material, Mesh, Shader, Viewport3D},
    Color, Device, Renderer, Surface
};
use std::{sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    device: Option<Device>,
    surface: Option<Surface>,
    renderer: Option<Renderer>,
    mesh: Option<Mesh>,
    material: Option<Material>,
    viewport: Option<Viewport3D>,
    start: Option<Instant>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_title("Rendra - Cube")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let device = Device::new().expect("Failed to initialize Device");
        let surface = Surface::builder(window.clone(), 800, 600)
            .depth(true)
            .build(&device)
            .expect("Failed to create Surface");
        let renderer = Renderer::new(&device);

        let mesh = Mesh::cube(1.0).build(&device).expect("Failed to build Mesh");

        let shader = Shader::builder()
            .build(&device)
            .expect("Failed to build Shader");
        let material = Material::builder(&shader)
            .tint(Color::GREEN)
            .build(&device, &surface)
            .expect("Failed to build Material");

        let mut viewport = Viewport3D::perspective(60f32.to_radians(), 0.1, 100.0);
        viewport.set_position(Vec3::new(0.0, 1.5, 4.0));
        viewport.look_at(Vec3::ZERO);

        self.window = Some(window);
        self.device = Some(device);
        self.surface = Some(surface);
        self.renderer = Some(renderer);
        self.mesh = Some(mesh);
        self.material = Some(material);
        self.viewport = Some(viewport);
        self.start = Some(Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent, ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(device), Some(surface)) = (self.device.as_ref(), self.surface.as_mut()) {
                    surface.resize(device, size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let (Some(renderer), Some(surface), Some(mesh), Some(material), Some(viewport), Some(start)) = (
                    self.renderer.as_mut(),
                    self.surface.as_mut(),
                    self.mesh.as_ref(),
                    self.material.as_ref(),
                    self.viewport.as_ref(),
                    self.start.as_ref(),
                ) else {
                    return;
                };

                let angle = start.elapsed().as_secs_f32();
                let transform = Mat4::from_rotation_y(angle);

                renderer.render(surface, Some(Color::GRAY), |frame| {
                        frame.draw(mesh, material, viewport, transform)
                    }).unwrap();
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