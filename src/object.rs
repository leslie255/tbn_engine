use std::sync::Arc;

use crate::{binding, camera::Camera, material::AsMaterial, mesh::AsMesh, surface::RenderPass};

use cgmath::*;

/// Contains the states needed to render an object.
#[derive(Debug, Clone)]
pub struct Object<Mesh: AsMesh, Material: AsMaterial> {
    mesh: Mesh,
    material: Arc<Material>,
    pipeline: wgpu::RenderPipeline,
    mesh_bind_group: wgpu::BindGroup,
    material_bind_group: wgpu::BindGroup,
}

impl<Mesh: AsMesh, Material: AsMaterial> Object<Mesh, Material> {
    pub fn create(
        device: &wgpu::Device,
        camera: &Camera,
        surface_format: wgpu::TextureFormat,
        mesh: Mesh,
        material: Arc<Material>,
    ) -> Self {
        let (wgpu_mesh_bind_group, mesh_bind_group_layout) =
            binding::create_wgpu_bind_group(device, &mesh);
        let (wgpu_material_bind_group, material_bind_group_layout) =
            binding::create_wgpu_bind_group(device, material.as_ref());
        let vertex_shader = Mesh::create_vertex_shader(device);
        let fragment_shader = Material::create_fragment_shader(device);
        let pipeline = {
            let bind_group_layouts: &[&wgpu::BindGroupLayout] = &[
                &binding::create_wgpu_bind_group_layout(device, camera),
                &mesh_bind_group_layout,
                &material_bind_group_layout,
            ];
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            });
            let vertex_buffer = mesh.vertex_buffer();
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_buffer.layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Material::blend_state(surface_format),
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
            mesh,
            material,
            pipeline,
            mesh_bind_group: wgpu_mesh_bind_group,
            material_bind_group: wgpu_material_bind_group,
        }
    }

    pub fn material(&self) -> &Arc<Material> {
        &self.material
    }

    pub fn material_ref(&self) -> &Material {
        &self.material
    }

    pub fn update_material(&mut self, device: &wgpu::Device, new_material: Arc<Material>) {
        let (new_bind_group, _) = binding::create_wgpu_bind_group(device, new_material.as_ref());
        self.material_bind_group = new_bind_group;
        self.material = new_material;
    }

    /// Set the model matrix.
    /// NOP if `Mesh::model_view` returns `None`.
    /// The camera must be the same one that's used in drawing this object later.
    pub fn set_model(&self, model: Matrix4<f32>, camera: &Camera, queue: &wgpu::Queue) {
        if let Some(model_view) = self.mesh.model_view() {
            model_view.write((camera.view_matrix() * model).into(), queue);
        }
    }

    pub fn draw(&self, render_pass: &mut RenderPass) {
        let wgpu_render_pass = render_pass.wgpu_render_pass_mut();
        wgpu_render_pass.set_pipeline(&self.pipeline);
        wgpu_render_pass.set_bind_group(1, &self.mesh_bind_group, &[]);
        wgpu_render_pass.set_bind_group(2, &self.material_bind_group, &[]);
        wgpu_render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer().slice(..));
        let index_buffer = self.mesh.index_buffer();
        wgpu_render_pass.set_index_buffer(index_buffer.slice(..), index_buffer.index_format());
        wgpu_render_pass.draw_indexed(0..index_buffer.length(), 0, 0..1);
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn mesh_mut(&mut self) -> &mut Mesh {
        &mut self.mesh
    }
}
