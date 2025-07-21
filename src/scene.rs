use std::{
    collections::{HashMap, hash_map},
    fmt::Debug,
    ops::DerefMut,
};

use cgmath::*;

use crate::{
    CameraBindGroup, CameraRef, Context, DepthStencilTextureFormat, MaterialRef, MeshRef,
    ObjectRef, SurfaceView, TextureFormat, binding,
};

#[derive(Debug, Clone)]
pub(crate) struct ObjectStorage {
    /// Its index in the scene's object list.
    pub(crate) id: u64,
    pub(crate) camera: CameraRef,
    pub(crate) mesh: MeshRef,
    pub(crate) material: MaterialRef,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) model: Matrix4<f32>,
    pub(crate) is_hidden: bool,
}

impl ObjectStorage {
    pub(crate) fn new(
        scene: &Scene,
        id: u64,
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
    object_indices: HashMap<u64, usize>,
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
            surface_color_format,
            surface_depth_stencil_format,
        }
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
    pub fn render(&self, context: &Context, surface: &SurfaceView) {
        // For more intuitive panic site if texture format mismatch happens:
        debug_assert!(surface.format() == self.surface_color_format);
        debug_assert!(surface.depth_stencil_format() == self.surface_depth_stencil_format);

        let mut render_pass = surface.render_pass(context.wgpu_device());

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
                .write(projection.into(), context.wgpu_queue());

            if let Some(model_view_uniform) = mesh.instance.model_view() {
                let model_view = camera.view_matrix() * object.model;
                let model_view_array: [[f32; 4]; 4] = model_view.into();
                context.wgpu_queue().write_buffer(
                    model_view_uniform,
                    0,
                    bytemuck::bytes_of(&model_view_array),
                );
            }

            let wgpu_render_pass = render_pass.wgpu_render_pass_mut();
            wgpu_render_pass.set_pipeline(&object.pipeline);
            wgpu_render_pass.set_bind_group(1, &mesh.wgpu_bind_group, &[]);
            wgpu_render_pass.set_bind_group(2, &material.wgpu_bind_group, &[]);
            wgpu_render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
            wgpu_render_pass.set_index_buffer(mesh.index_buffer().slice(..), mesh.index_format);
            wgpu_render_pass.draw_indexed(0..mesh.index_buffer_length(), 0, 0..1);
        }

        render_pass.finish(context.wgpu_queue());
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
