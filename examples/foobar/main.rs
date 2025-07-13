#![allow(linker_messages)]

use std::{sync::Arc, time::SystemTime};

use tbn_engine::*;

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

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    window_surface: WindowSurface,
    scene: Scene,
    ground: ObjectId,
    cube_0: ObjectId,
    cube_1: ObjectId,
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

        let y = 180.0;
        let camera = Camera::create(
            &device,
            point3(0.0, y, 0.0),
            vec3(0.0, 1.0, 0.0),
            CameraDirection::LookAt(point3(0.0, y, 0.0)),
            Deg(50.0),
            1.0,
            1000.0,
        );

        let mut scene = Scene::new(
            camera,
            window_surface.format(),
            window_surface.depth_stencil_format(),
        );

        let cube_0_material = scene.add_material(&device, &{
            let material = UniformFill::create(&device);
            material
                .fill_color
                .write(Rgba::new(0.7, 0.4, 1.0, 1.0), &queue);
            material
        });
        let cube_0_mesh = scene.add_mesh(
            &device,
            Arc::new(Mesh3D::create(&device, &CUBE_VERTICES, &CUBE_INDICIES)),
        );
        let cube_0 = scene.add_object(&device, cube_0_mesh, cube_0_material);

        let image = test_image();
        let texture = Texture2d::create_init(
            &device,
            &queue,
            image.size,
            image.format,
            image.data.as_ref(),
        );
        let cube_1_material = scene.add_material(&device, &{
            Textured::create(
                &device,
                texture.view(Default::default()),
                Sampler::create(
                    &device,
                    wgpu::AddressMode::ClampToEdge,
                    wgpu::FilterMode::Linear,
                    wgpu::FilterMode::Linear,
                ),
            )
        });
        let cube_1_mesh = scene.add_mesh(
            &device,
            Arc::new(Mesh3D::create(&device, &CUBE_VERTICES, &CUBE_INDICIES)),
        );
        let cube_1 = scene.add_object(&device, cube_1_mesh, cube_1_material);

        let ground_material = scene.add_material(&device, &{
            let material = SdfCircle::create(&device);
            material
                .fill_color
                .write(Rgba::new(0.5, 0.5, 0.5, 1.0), &queue);
            material
        });
        let ground_mesh = scene.add_mesh(&device, Arc::new(Quad::create(&device)));
        let ground = scene.add_object(&device, ground_mesh, ground_material);

        State {
            window,
            device,
            queue,
            window_surface,
            scene,
            ground,
            cube_0,
            cube_1,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.window_surface.resized(new_size, &self.device);
    }

    fn render(&mut self) {
        self.window_surface.frame(|surface| {
            let t = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            // Update camera position.
            self.scene.camera_mut().position.x = (f64::cos(t) as f32) * 400.0;
            self.scene.camera_mut().position.z = (f64::sin(t) as f32) * 400.0;
            self.scene
                .camera_mut()
                .update_projection_uniform(surface.size_f32(), &self.queue);

            // Ground.
            let camera_far = self.scene.camera().far;
            self.scene.set_model(
                &self.queue,
                self.ground,
                Matrix4::from_scale(camera_far * 2.0)
                    * Matrix4::from_translation(vec3(-0.5, 0.0, -0.5))
                    * Matrix4::from_angle_x(Deg(90.0)),
            );

            // Cube 0.
            let cube_0_size = 100.0;
            self.scene.set_model(
                &self.queue,
                self.cube_0,
                Matrix4::from_translation(vec3(-120.0, cube_0_size + 100.0, 0.0))
                    * Matrix4::from_angle_x(Deg(45.0))
                    * Matrix4::from_angle_y(Deg(30.0))
                    * Matrix4::from_scale(cube_0_size)
                    * Matrix4::from_translation([-0.5; 3].into()),
            );

            // Cube 1.
            let cube_1_size = 80.0;
            self.scene.set_model(
                &self.queue,
                self.cube_1,
                Matrix4::from_translation(vec3(120.0, cube_1_size + 80.0, 0.0))
                    * Matrix4::from_angle_x(Deg(-12.0))
                    * Matrix4::from_angle_z(Deg(40.0))
                    * Matrix4::from_scale(cube_1_size)
                    * Matrix4::from_translation([-0.5; 3].into()),
            );

            self.scene.render(&self.device, &self.queue, &surface);
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
