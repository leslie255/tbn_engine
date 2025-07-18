use std::{
    collections::{HashMap, hash_map},
    fmt::{self, Debug},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use cgmath::*;

use crate::{
    AsMaterial, AsMesh, Camera, CameraBindGroup, DepthStencilTextureFormat, DynMesh, SurfaceView,
    TextureFormat, binding,
};

// TODO: Perhaps use a third-party, `Weak`-less `Arc`.
macro_rules! define_ref_type {
    ($T:ident, $Storage:ty $(,)?) => {
        #[allow(dead_code)]
        #[derive(Debug, Clone)]
        pub struct $T {
            storage: Arc<Mutex<$Storage>>,
        }

        impl $T {
            fn new(storage: $Storage) -> Self {
                Self {
                    storage: Arc::new(Mutex::new(storage)),
                }
            }

            #[track_caller]
            fn lock(&self) -> impl DerefMut<Target = $Storage> {
                self.storage.lock().unwrap()
            }
        }
    };
}

define_ref_type!(MeshRef, MeshStorage);
define_ref_type!(MaterialRef, MaterialStorage);
define_ref_type!(ObjectRef, ObjectStorage);
define_ref_type!(CameraRef, Camera);

impl ObjectRef {
    pub fn set_is_hidden(&self, is_hidden: bool) {
        self.lock().is_hidden = is_hidden;
    }

    pub fn get_is_hidden(&self) -> bool {
        self.lock().is_hidden
    }
}

impl CameraRef {
    pub fn with_mut<T>(&self, f: impl FnOnce(&mut Camera) -> T) -> T {
        // Operates on a copy of the camera in case user tries to render while inside the closure.
        let mut camera_copy = self.lock().clone();
        let result = f(&mut camera_copy);
        *self.lock() = camera_copy;
        result
    }
}

#[derive(Clone)]
struct MeshStorage {
    instance: Arc<dyn DynMesh>,
    vertex_shader: wgpu::ShaderModule,
    wgpu_bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    index_format: wgpu::IndexFormat,
}

impl Debug for MeshStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("MeshStorage")
            .field("vertex_shader", &self.vertex_shader)
            .field("wgpu_bind_group", &self.wgpu_bind_group)
            .field("bind_group_layout", &self.bind_group_layout)
            .field("vertex_buffer_layout", &self.vertex_buffer_layout)
            .finish_non_exhaustive()
    }
}

impl MeshStorage {
    fn new<Mesh: AsMesh>(device: &wgpu::Device, mesh_instance: Arc<Mesh>) -> Self {
        let (wgpu_bind_group, bind_group_layout) =
            binding::create_wgpu_bind_group(device, Arc::deref(&mesh_instance));
        let vertex_buffer_layout = mesh_instance.vertex_buffer().layout();
        let index_format = mesh_instance.index_buffer().index_format();
        Self {
            instance: mesh_instance.as_arc_dyn(),
            vertex_shader: Mesh::create_vertex_shader(device),
            wgpu_bind_group,
            bind_group_layout,
            vertex_buffer_layout,
            index_format,
        }
    }

    fn vertex_buffer(&self) -> &wgpu::Buffer {
        self.instance.vertex_buffer()
    }

    fn index_buffer(&self) -> &wgpu::Buffer {
        self.instance.index_buffer()
    }

    fn index_buffer_length(&self) -> u32 {
        self.instance.index_buffer_length()
    }
}

#[derive(Debug, Clone)]
struct MaterialStorage {
    fragment_shader: wgpu::ShaderModule,
    wgpu_bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    blend_state: Option<wgpu::BlendState>,
}

impl MaterialStorage {
    fn new<Material: AsMaterial>(
        device: &wgpu::Device,
        surface_format: TextureFormat,
        material_instance: &Material,
    ) -> Self {
        let (wgpu_bind_group, bind_group_layout) =
            binding::create_wgpu_bind_group(device, material_instance);
        Self {
            fragment_shader: Material::create_fragment_shader(device),
            wgpu_bind_group,
            bind_group_layout,
            blend_state: Material::blend_state(surface_format),
        }
    }
}

#[derive(Debug, Clone)]
struct ObjectStorage {
    /// Its index in the scene's object list.
    id: u32,
    camera: CameraRef,
    mesh: MeshRef,
    material: MaterialRef,
    pipeline: wgpu::RenderPipeline,
    model: Matrix4<f32>,
    is_hidden: bool,
}

impl ObjectStorage {
    fn new(
        scene: &Scene,
        id: u32,
        device: &wgpu::Device,
        camera: CameraRef,
        mesh: MeshRef,
        material: MaterialRef,
    ) -> Self {
        let mesh_storage = mesh.lock();
        let material_storage = material.lock();
        let pipeline = {
            let bind_group_layouts: &[&wgpu::BindGroupLayout] = &[
                &binding::create_wgpu_bind_group_layout(device, &scene.camera_bind_group),
                &mesh_storage.bind_group_layout,
                &material_storage.bind_group_layout,
            ];
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &mesh_storage.vertex_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[mesh_storage.vertex_buffer_layout.clone()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &material_storage.fragment_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: scene.surface_color_format.into(),
                        blend: material_storage.blend_state,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };
        drop((mesh_storage, material_storage));
        Self {
            id,
            camera,
            mesh,
            material,
            pipeline,
            model: Matrix4::identity(),
            is_hidden: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    camera_bind_group: CameraBindGroup,
    camera_wgpu_bind_group: wgpu::BindGroup,
    objects: Vec<Option<ObjectRef>>,
    object_indices: HashMap<u32, usize>,
    object_id_counter: u32,
    #[expect(dead_code)]
    camera_wgpu_bind_group_layout: wgpu::BindGroupLayout,
    surface_color_format: TextureFormat,
    surface_depth_stencil_format: DepthStencilTextureFormat,
}

impl Scene {
    pub fn new(
        device: &wgpu::Device,
        surface_color_format: TextureFormat,
        surface_depth_stencil_format: DepthStencilTextureFormat,
    ) -> Self {
        let camera_bind_group = CameraBindGroup::create(device);
        let (camera_wgpu_bind_group, camera_wgpu_bind_group_layout) =
            binding::create_wgpu_bind_group(device, &camera_bind_group);
        Self {
            camera_bind_group,
            camera_wgpu_bind_group,
            camera_wgpu_bind_group_layout,
            objects: Vec::new(),
            object_indices: HashMap::new(),
            object_id_counter: 0,
            surface_color_format,
            surface_depth_stencil_format,
        }
    }

    pub fn create_mesh(&mut self, device: &wgpu::Device, instance: Arc<impl AsMesh>) -> MeshRef {
        let mesh_storage = MeshStorage::new(device, instance);
        MeshRef::new(mesh_storage)
    }

    pub fn create_material(
        &mut self,
        device: &wgpu::Device,
        instance: &impl AsMaterial,
    ) -> MaterialRef {
        let material_storage = MaterialStorage::new(device, self.surface_color_format, instance);
        MaterialRef::new(material_storage)
    }

    /// Create an object and add it into the list of objects for rendering.
    pub fn create_object(
        &mut self,
        device: &wgpu::Device,
        camera: CameraRef,
        mesh: MeshRef,
        material: MaterialRef,
    ) -> ObjectRef {
        let id = self.object_id_counter;
        self.object_id_counter += 1;
        let object_storage = ObjectStorage::new(self, id, device, camera, mesh, material);
        let object = ObjectRef::new(object_storage);
        self.add_object(object.clone());
        object
    }

    pub fn create_camera(&mut self, camera: Camera) -> CameraRef {
        CameraRef::new(camera)
    }

    /// Add object to the list of object for rendering.
    ///
    /// # Panics
    ///
    /// - if object was aready in the list
    pub fn add_object(&mut self, object: ObjectRef) {
        let id = { object.lock().id };
        let index = self.objects.len();
        match self.object_indices.entry(id) {
            hash_map::Entry::Occupied(_) => panic!("object was already in the list"),
            hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(index);
            }
        }
        self.objects.push(Some(object.clone()));
    }

    /// Removes an object from the list of objects for rendering.
    ///
    /// # Panics
    ///
    /// - if object wasn't in the list
    pub fn remove_object(&mut self, object: &ObjectRef) {
        let object_id = { object.lock().id };
        let &index = self
            .object_indices
            .get(&object_id)
            .unwrap_or_else(|| panic!("object wasn't in the list"));
        self.objects[index] = None;
    }

    fn objects(&self) -> impl Iterator<Item = impl DerefMut<Target = ObjectStorage>> {
        self.objects
            .iter()
            .filter_map(Option::as_ref)
            .map(ObjectRef::lock)
    }

    /// Renders the scene onto the surface with a camera.
    /// TODO: perhaps make cameras also registerable, similar to mesh, material and object
    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &SurfaceView) {
        // For more intuitive panic site if texture format mismatch happens:
        debug_assert!(surface.format() == self.surface_color_format);
        debug_assert!(surface.depth_stencil_format() == self.surface_depth_stencil_format);

        let mut render_pass = surface.render_pass(device);

        render_pass.set_bind_group(0, &self.camera_wgpu_bind_group, &[]);

        for object in self.objects() {
            let camera = object.camera.lock();
            let mesh = object.mesh.lock();
            let material = object.material.lock();
            if object.is_hidden {
                continue;
            }

            // Updates projection uniform.
            let projection = camera.projection_matrix(surface.size_f32());
            self.camera_bind_group
                .projection
                .write(projection.into(), queue);

            if let Some(model_view_uniform) = mesh.instance.model_view() {
                let model_view = camera.view_matrix() * object.model;
                let model_view_array: [[f32; 4]; 4] = model_view.into();
                queue.write_buffer(model_view_uniform, 0, bytemuck::bytes_of(&model_view_array));
            }
            let wgpu_render_pass = render_pass.wgpu_render_pass_mut();
            wgpu_render_pass.set_pipeline(&object.pipeline);
            wgpu_render_pass.set_bind_group(1, &mesh.wgpu_bind_group, &[]);
            wgpu_render_pass.set_bind_group(2, &material.wgpu_bind_group, &[]);
            wgpu_render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
            wgpu_render_pass.set_index_buffer(mesh.index_buffer().slice(..), mesh.index_format);
            wgpu_render_pass.draw_indexed(0..mesh.index_buffer_length(), 0, 0..1);
        }

        render_pass.finish(queue);
    }

    /// Set the model matrix for an object's mesh.
    /// NOP for objects that doesn't use traditional model-view matrix (object with a mesh that
    /// returns `None` for `AsMesh::model_view`).
    pub fn set_object_model(&self, object: &ObjectRef, model: Matrix4<f32>) {
        let mut object = object.lock();
        object.model = model;
    }

    /// Set whether an object is hidden.
    pub fn set_object_is_hidden(&self, object: &ObjectRef, is_hidden: bool) {
        let mut object = object.lock();
        object.is_hidden = is_hidden;
    }
}
