#![allow(linker_messages)]

use std::{sync::Arc, time::SystemTime};

use binding::{Sampler2d, Texture2d};
use buffers::Vertex3dUV;
use camera::{Camera, CameraDirection};
use cgmath::*;
use color::Rgba;
use mesh::{Mesh3D, Quad};
use object::Object;
use pollster::FutureExt as _;
use surface::{Surface, SurfaceView, WindowSurface};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub mod binding;
pub mod buffers;
pub mod camera;
pub mod color;
pub mod material;
pub mod mesh;
pub mod object;
pub mod surface;

struct Image<Data: AsRef<[u8]>> {
    format: wgpu::TextureFormat,
    size: Vector2<u32>,
    data: Data,
}

fn test_image() -> Image<impl AsRef<[u8]>> {
    Image {
        format: wgpu::TextureFormat::Rgba8Unorm,
        size: vec2(256, 256),
        data: include_bytes!("../image.bin"),
    }
}

const CUBE_VERTICES: [Vertex3dUV; 24] = [
    // South (+Z)
    Vertex3dUV::new([0., 0., 1.], [0.0, 1.0]),
    Vertex3dUV::new([1., 0., 1.], [1.0, 1.0]),
    Vertex3dUV::new([1., 1., 1.], [1.0, 0.0]),
    Vertex3dUV::new([0., 1., 1.], [0.0, 0.0]),
    // North (-Z)
    Vertex3dUV::new([0., 0., 0.], [1.0, 1.0]),
    Vertex3dUV::new([0., 1., 0.], [1.0, 0.0]),
    Vertex3dUV::new([1., 1., 0.], [0.0, 0.0]),
    Vertex3dUV::new([1., 0., 0.], [0.0, 1.0]),
    // East (+X)
    Vertex3dUV::new([1., 0., 0.], [1.0, 1.0]),
    Vertex3dUV::new([1., 1., 0.], [1.0, 0.0]),
    Vertex3dUV::new([1., 1., 1.], [0.0, 0.0]),
    Vertex3dUV::new([1., 0., 1.], [0.0, 1.0]),
    // West (-X)
    Vertex3dUV::new([0., 1., 0.], [0.0, 0.0]),
    Vertex3dUV::new([0., 0., 0.], [0.0, 1.0]),
    Vertex3dUV::new([0., 0., 1.], [1.0, 1.0]),
    Vertex3dUV::new([0., 1., 1.], [1.0, 0.0]),
    // Up (+Y)
    Vertex3dUV::new([1., 1., 0.], [0.0, 1.0]),
    Vertex3dUV::new([0., 1., 0.], [1.0, 1.0]),
    Vertex3dUV::new([0., 1., 1.], [1.0, 0.0]),
    Vertex3dUV::new([1., 1., 1.], [0.0, 0.0]),
    // Down (-Y)
    Vertex3dUV::new([0., 0., 0.], [0.0, 1.0]),
    Vertex3dUV::new([1., 0., 0.], [1.0, 1.0]),
    Vertex3dUV::new([1., 0., 1.], [1.0, 0.0]),
    Vertex3dUV::new([0., 0., 1.], [0.0, 0.0]),
];

const CUBE_INDICIES: [u32; 36] = [
    0, 1, 2, 2, 3, 0, // South (+Z)
    4, 5, 6, 6, 7, 4, // North (-Z)
    8, 9, 10, 10, 11, 8, // East (+X)
    12, 13, 14, 14, 15, 12, // West (-X)
    16, 17, 18, 18, 19, 16, // Up (+Y)
    20, 21, 22, 22, 23, 20, // Down (-Y)
];

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    window_surface: WindowSurface,
    /// The render result before post-processing.
    scene_surface: Surface,
    /// The quad used for rendering post-processed textures.
    quad: Object<Quad, material::Textured>,
    camera: Camera,
    test_quad: Object<Quad, material::Textured>,
    test_cube: Object<Mesh3D, material::UniformFill>,
}

impl State {
    fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .block_on()
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .block_on()
            .unwrap();

        let window_surface = WindowSurface::new(Arc::clone(&window), &instance, &adapter, &device);

        let staging_surface = Self::create_staging_surface(&device, window_surface.physical_size());

        let camera = Camera::create(
            &device,
            point3(0.0, 0.0, -1000.0),
            vec3(0.0, 1.0, 0.0),
            CameraDirection::LookAt(point3(0.0, 0.0, 0.0)),
            Deg(70.0),
            1.0,
            100000.0,
        );

        let staging_quad_material = Arc::new(material::Textured::create(
            &device,
            staging_surface.color_texture().view(Default::default()),
            Sampler2d::create(
                &device,
                wgpu::AddressMode::ClampToEdge,
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
            ),
        ));
        let staging_quad = Object::create(
            &device,
            &camera,
            window_surface.format(),
            Quad::create(&device),
            staging_quad_material,
        );

        let test_quad = {
            let image = test_image();
            let texture = Texture2d::create_init(
                &device,
                &queue,
                image.size,
                image.format,
                image.data.as_ref(),
            );
            let texture_view = texture.view(Default::default());
            let sampler = Sampler2d::create(
                &device,
                wgpu::AddressMode::ClampToEdge,
                wgpu::FilterMode::Nearest,
                wgpu::FilterMode::Nearest,
            );
            let material = Arc::new(material::Textured::create(&device, texture_view, sampler));
            let mesh = Quad::create(&device);
            Object::create(&device, &camera, staging_surface.format(), mesh, material)
        };

        let test_cube = {
            let material = Arc::new(material::UniformFill::create(&device));
            material
                .fill_color
                .write(Rgba::new(0.7, 0.4, 1.0, 1.0), &queue);
            let mesh = Mesh3D::create(&device, &CUBE_VERTICES, &CUBE_INDICIES);
            Object::create(&device, &camera, staging_surface.format(), mesh, material)
        };

        State {
            window,
            device,
            queue,
            window_surface,
            scene_surface: staging_surface,
            quad: staging_quad,
            camera,
            test_quad,
            test_cube,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn create_staging_surface(device: &wgpu::Device, size: Vector2<u32>) -> Surface {
        Surface::create(device, size, wgpu::TextureFormat::Rgba16Float)
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.window_surface.resized(new_size, &self.device);
        self.scene_surface =
            Self::create_staging_surface(&self.device, self.window_surface.physical_size());
    }

    fn draw(&mut self, surface: SurfaceView) {
        let mut render_pass = surface.render_pass(&self.device);

        let t = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        // Update camera position.
        self.camera.position = point3(f64::cos(t) as f32, 0.0, f64::sin(t) as f32) * 400.0;
        self.camera
            .update_projection_uniform(surface.size_f32(), &self.queue);
        render_pass.use_camera(&self.camera);

        // Draw the test quad.
        let size = 100.0;
        let position = vec3(-size / 2.0, -size / 2.0, 0.0);
        let model = Matrix4::from_translation(position) * Matrix4::from_scale(size);
        self.test_quad.set_model(model, &self.camera, &self.queue);
        self.test_quad.draw(&mut render_pass);

        // Draw the cube.
        let size = 100.0;
        let position = vec3(240.0 - size / 2.0, -size / 2.0, -size / 2.0);
        let model = Matrix4::from_translation(position)
            * Matrix4::from_angle_x(Deg(45.0))
            * Matrix4::from_angle_y(Deg(30.0))
            * Matrix4::from_scale(size);
        self.test_cube.set_model(model, &self.camera, &self.queue);
        self.test_cube.draw(&mut render_pass);

        render_pass.finish(&self.queue);
    }

    fn render(&mut self) {
        self.draw(self.scene_surface.view());
        self.window_surface.frame(|surface| {
            let mut render_pass = surface.render_pass(&self.device);

            let camera = Camera::create(
                &self.device,
                point3(0.0, 0.0, 10.0),
                vec3(0.0, 1.0, 0.0),
                CameraDirection::LookTo(vec3(0.0, 0.0, -1.0)),
                Rad(0.0),
                -1.0,
                1.0,
            );

            render_pass.use_camera(&camera);

            let material = material::Textured::create(
                &self.device,
                self.scene_surface.color_texture().view(Default::default()),
                Sampler2d::create(
                    &self.device,
                    wgpu::AddressMode::ClampToEdge,
                    wgpu::FilterMode::Linear,
                    wgpu::FilterMode::Linear,
                ),
            );
            material.gamma().write(2.2, &self.queue);
            self.quad.update_material(&self.device, Arc::new(material));
            let model = Matrix4::from_translation(vec3(-1.0, -1.0, 0.0)) * Matrix4::from_scale(2.0);
            self.quad
                .mesh()
                .model_view
                .write(model.into(), &self.queue);
            self.quad.draw(&mut render_pass);

            render_pass.finish(&self.queue);
        });
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = State::new(Arc::clone(&window));

        event_loop.set_control_flow(ControlFlow::Poll);

        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                state.window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
