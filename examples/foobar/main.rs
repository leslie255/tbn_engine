#![allow(linker_messages)]

use std::{sync::Arc, time::SystemTime};

use tbn_engine::binding::{Sampler2d, TextureView2d};
use tbn_engine::buffers::Vertex3dUV;
use tbn_engine::camera::{Camera, CameraDirection};
use tbn_engine::color::Rgba;
use tbn_engine::material::{self, AsMaterial};
use tbn_engine::mesh::{Mesh3D, Quad};
use tbn_engine::object::Object;
use tbn_engine::surface::{Surface, SurfaceView, WindowSurface};

use pollster::FutureExt as _;

use cgmath::*;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[allow(dead_code)]
struct Image<Data: AsRef<[u8]>> {
    format: wgpu::TextureFormat,
    size: Vector2<u32>,
    data: Data,
}

#[allow(dead_code)]
fn test_image() -> Image<impl AsRef<[u8]>> {
    Image {
        format: wgpu::TextureFormat::Rgba8Unorm,
        size: vec2(256, 256),
        data: include_bytes!("./image.bin"),
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

struct PostProcessMaterial {
    sampler: Sampler2d,
    color_texture: TextureView2d,
    depth_texture: TextureView2d,
}

tbn_engine::impl_as_bind_group! {
    PostProcessMaterial {
        0 => sampler,
        1 => color_texture,
        2 => depth_texture,
    }
}

impl PostProcessMaterial {
    pub fn create(device: &wgpu::Device, surface: &Surface) -> Self {
        let color_texture = surface.color_texture().view(Default::default());
        let depth_texture = surface
            .depth_stencil_texture()
            .view(wgpu::TextureSampleType::Depth);
        let sampler = Sampler2d::create(
            device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Linear,
            wgpu::FilterMode::Linear,
        );
        Self {
            sampler,
            color_texture,
            depth_texture,
        }
    }
}

impl AsMaterial for PostProcessMaterial {
    fn create_fragment_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("./postprocess.wgsl").into()),
        })
    }
}

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    window_surface: WindowSurface,
    /// The render result before post-processing.
    render_result_0: Surface,
    /// The quad used for rendering post-processed textures.
    postprocess_quad: Object<Quad, PostProcessMaterial>,
    camera: Camera,
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

        let render_result_0 = Self::create_staging_surface(&device, window_surface.physical_size());

        let camera = Camera::create(
            &device,
            point3(0.0, 0.0, -1000.0),
            vec3(0.0, 1.0, 0.0),
            CameraDirection::LookAt(point3(0.0, 0.0, 0.0)),
            Deg(50.0),
            1.0,
            100000.0,
        );

        let staging_quad_material =
            Arc::new(PostProcessMaterial::create(&device, &render_result_0));
        let postprocess_quad = Object::create(
            &device,
            &camera,
            window_surface.format(),
            Quad::create(&device),
            staging_quad_material,
        );

        let test_cube = {
            let material = Arc::new(material::UniformFill::create(&device));
            material
                .fill_color
                .write(Rgba::new(0.7, 0.4, 1.0, 1.0), &queue);
            let mesh = Mesh3D::create(&device, &CUBE_VERTICES, &CUBE_INDICIES);
            Object::create(&device, &camera, render_result_0.format(), mesh, material)
        };

        State {
            window,
            device,
            queue,
            window_surface,
            render_result_0,
            postprocess_quad,
            camera,
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
        self.render_result_0 =
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

        // Draw the cube.
        let size = 100.0;
        let model = Matrix4::from_angle_x(Deg(45.0))
            * Matrix4::from_angle_y(Deg(30.0))
            * Matrix4::from_translation(-0.5 * vec3(size, size, size))
            * Matrix4::from_scale(size);
        self.test_cube.set_model(model, &self.camera, &self.queue);
        self.test_cube.draw(&mut render_pass);

        render_pass.finish(&self.queue);
    }

    fn render(&mut self) {
        self.draw(self.render_result_0.view());
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
            camera.update_projection_uniform(self.window_surface.physical_size_f32(), &self.queue);

            render_pass.use_camera(&camera);

            let material = PostProcessMaterial::create(&self.device, &self.render_result_0);
            self.postprocess_quad
                .update_material(&self.device, Arc::new(material));
            let size = self.window_surface.physical_size_f32();
            let size_half = size / 2.0;
            let model = Matrix4::from_translation(-size_half.extend(0.0))
                * Matrix4::from_nonuniform_scale(size.x, size.y, 1.0);
            self.postprocess_quad
                .mesh()
                .model_view
                .write(model.into(), &self.queue);
            self.postprocess_quad.draw(&mut render_pass);

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
