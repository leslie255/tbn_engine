use std::{ops::Deref as _, sync::Arc};

use index_vec::IndexVec;

use cgmath::*;

use crate::{AsMaterial, AsMesh, Camera, DynMesh, SurfaceView, binding};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshId(pub usize);
impl index_vec::Idx for MeshId {
    fn from_usize(idx: usize) -> Self {
        Self(idx)
    }
    fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId(pub usize);
impl index_vec::Idx for MaterialId {
    fn from_usize(idx: usize) -> Self {
        Self(idx)
    }
    fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(pub usize);
impl index_vec::Idx for ObjectId {
    fn from_usize(idx: usize) -> Self {
        Self(idx)
    }
    fn index(self) -> usize {
        self.0
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

impl std::fmt::Debug for MeshStorage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        surface_format: wgpu::TextureFormat,
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
    mesh_id: MeshId,
    material_id: MaterialId,
    pipeline: wgpu::RenderPipeline,
    is_hidden: bool,
}

impl ObjectStorage {
    fn new(scene: &Scene, device: &wgpu::Device, mesh_id: MeshId, material_id: MaterialId) -> Self {
        let mesh = scene.mesh(mesh_id);
        let material = scene.material(material_id);
        let pipeline = {
            let bind_group_layouts: &[&wgpu::BindGroupLayout] = &[
                &binding::create_wgpu_bind_group_layout(device, &scene.camera),
                &mesh.bind_group_layout,
                &material.bind_group_layout,
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
                    module: &mesh.vertex_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[mesh.vertex_buffer_layout.clone()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &material.fragment_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: scene.surface_color_format,
                        blend: material.blend_state,
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
        Self {
            mesh_id,
            material_id,
            pipeline,
            is_hidden: false,
        }
    }
}

// /// Can be used to create a camera's bind group layout.
// /// Panics if tried to create the actual bind group.
// struct PhantomCamera;
// impl AsBindGroup for PhantomCamera {
//     fn bind_group_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
//         vec![wgpu::BindGroupLayoutEntry {
//             binding: 0,
//             visibility: wgpu::ShaderStages::all(),
//             ty: wgpu::BindingType::Buffer {
//                 ty: wgpu::BufferBindingType::Uniform,
//                 has_dynamic_offset: false,
//                 min_binding_size: None,
//             },
//             count: None,
//         }]
//     }
//     fn bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
//         panic!()
//     }
// }

#[derive(Debug, Clone)]
pub struct Scene {
    mesh_registry: IndexVec<MeshId, MeshStorage>,
    material_registry: IndexVec<MaterialId, MaterialStorage>,
    object_registry: IndexVec<ObjectId, ObjectStorage>,
    camera: Camera,
    surface_color_format: wgpu::TextureFormat,
    surface_depth_stencil_format: wgpu::TextureFormat,
}

impl Scene {
    pub fn new(
        camera: Camera,
        surface_color_format: wgpu::TextureFormat,
        surface_depth_stencil_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            mesh_registry: IndexVec::new(),
            material_registry: IndexVec::new(),
            object_registry: IndexVec::new(),
            camera,
            surface_color_format,
            surface_depth_stencil_format,
        }
    }

    fn mesh(&self, mesh_id: MeshId) -> &MeshStorage {
        &self.mesh_registry[mesh_id]
    }

    fn material(&self, material_id: MaterialId) -> &MaterialStorage {
        &self.material_registry[material_id]
    }

    fn object(&self, object_id: ObjectId) -> &ObjectStorage {
        &self.object_registry[object_id]
    }

    pub fn add_mesh(&mut self, device: &wgpu::Device, mesh_instance: Arc<impl AsMesh>) -> MeshId {
        self.mesh_registry
            .push(MeshStorage::new(device, mesh_instance))
    }

    pub fn add_material(
        &mut self,
        device: &wgpu::Device,
        material_instance: &impl AsMaterial,
    ) -> MaterialId {
        self.material_registry.push(MaterialStorage::new(
            device,
            self.surface_color_format,
            material_instance,
        ))
    }

    pub fn add_object(
        &mut self,
        device: &wgpu::Device,
        mesh: MeshId,
        material: MaterialId,
    ) -> ObjectId {
        let object = ObjectStorage::new(self, device, mesh, material);
        self.object_registry.push(object)
    }

    /// Skips if object is hidden.
    fn draw_object(&self, object_id: ObjectId, render_pass: &mut crate::RenderPass) {
        let object = &self.object(object_id);
        // These two lines are delibrately placed before the `is_hidden` check.
        // This is to make sure that had an invalid ID be produced somewhere, the panic site is
        // closer to the source.
        let mesh = self.mesh(object.mesh_id);
        let material = self.material(object.material_id);
        if object.is_hidden {
            return;
        }
        let wgpu_render_pass = render_pass.wgpu_render_pass_mut();
        wgpu_render_pass.set_pipeline(&object.pipeline);
        wgpu_render_pass.set_bind_group(1, &mesh.wgpu_bind_group, &[]);
        wgpu_render_pass.set_bind_group(2, &material.wgpu_bind_group, &[]);
        wgpu_render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
        wgpu_render_pass.set_index_buffer(mesh.index_buffer().slice(..), mesh.index_format);
        wgpu_render_pass.draw_indexed(0..mesh.index_buffer_length(), 0, 0..1);
    }

    /// Renders the scene onto the surface with a camera.
    /// TODO: perhaps make cameras also registerable, similar to mesh, material and object
    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &SurfaceView) {
        self.camera
            .update_projection_uniform(surface.size_f32(), queue);
        assert!(surface.format() == self.surface_color_format);
        assert!(surface.depth_stencil_format() == self.surface_depth_stencil_format);
        let mut render_pass = surface.render_pass(device);
        render_pass.set_bind_group(0, self.camera.wgpu_bind_group(), &[]);
        for object_id in self.object_registry.indices() {
            self.draw_object(object_id, &mut render_pass);
        }
        render_pass.finish(queue);
    }

    /// Set the model matrix for an object.
    /// NOP for objects that doesn't use traditional model-view matrix (object with a mesh that
    /// returns `None` for `AsMesh::model_view`).
    pub fn set_model(&self, queue: &wgpu::Queue, object_id: ObjectId, model: Matrix4<f32>) {
        let model_view = self.camera.view_matrix() * model;
        let model_view_array: [[f32; 4]; 4] = model_view.into();
        let object = self.object(object_id);
        let mesh = self.mesh(object.mesh_id);
        if let Some(model_view_buffer) = mesh.instance.model_view() {
            queue.write_buffer(model_view_buffer, 0, bytemuck::bytes_of(&model_view_array));
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}
