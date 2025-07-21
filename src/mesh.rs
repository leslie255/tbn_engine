use core::fmt;
use std::{fmt::Debug, ops::Deref as _, sync::Arc};

use crate::{
    binding, impl_as_bind_group, AsBindGroup, Index, IndexBuffer, UniformBuffer, Vertex, Vertex2d, Vertex3dUV, VertexBuffer
};

use cgmath::*;

pub trait AsMesh: AsBindGroup + Sized + 'static {
    type Vertex: Vertex;
    type Index: Index;

    /// TODO: Perhaps make an abstraction for shaders similar to bevy's `ShaderRef`.
    fn create_vertex_shader(device: &wgpu::Device) -> wgpu::ShaderModule;

    fn vertex_buffer(&self) -> &VertexBuffer<Self::Vertex>;

    fn index_buffer(&self) -> &IndexBuffer<Self::Index>;

    /// The model-view matrix.
    /// `None` if this mesh doesn't use a traditional model-view matrix setup.
    fn model_view(&self) -> Option<&UniformBuffer<[[f32; 4]; 4]>>;

    fn as_arc_dyn(self: Arc<Self>) -> Arc<dyn DynMesh> {
        self
    }
}

/// Type-erased, dyn-compatible form of the mesh trait.
pub trait DynMesh {
    fn vertex_buffer(&self) -> &wgpu::Buffer;
    fn index_buffer(&self) -> &wgpu::Buffer;
    fn index_buffer_length(&self) -> u32;
    fn model_view(&self) -> Option<&wgpu::Buffer>;
}

impl<T: AsMesh> DynMesh for T {
    fn vertex_buffer(&self) -> &wgpu::Buffer {
        AsMesh::vertex_buffer(self).wgpu_buffer()
    }

    fn index_buffer(&self) -> &wgpu::Buffer {
        AsMesh::index_buffer(self).wgpu_buffer()
    }

    fn index_buffer_length(&self) -> u32 {
        AsMesh::index_buffer(self).length()
    }

    fn model_view(&self) -> Option<&wgpu::Buffer> {
        AsMesh::model_view(self).map(UniformBuffer::wgpu_buffer)
    }
}

pub mod meshes {
    use crate::Context;

    use super::*;

    /// A simple quad shape.
    ///
    /// ```txt
    ///  (0, 1)   (1, 1)
    /// +--------+
    /// |        |
    /// |        |
    /// |        |
    /// |        |
    /// +--------+
    ///  (0, 0)   (1, 0)
    /// ```
    #[derive(Debug, Clone)]
    pub struct Quad {
        vertex_buffer: VertexBuffer<Vertex2d>,
        index_buffer: IndexBuffer<u16>,
        /// The model-view matrix.
        pub model_view: UniformBuffer<[[f32; 4]; 4]>,
        /// Apply a transform on the UV coordinates.
        pub uv_transform: UniformBuffer<[[f32; 4]; 4]>,
    }

    impl_as_bind_group! {
        Quad {
            0 => model_view,
            1 => uv_transform,
        }
    }

    impl AsMesh for Quad {
        type Vertex = Vertex2d;
        type Index = u16;

        fn create_vertex_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/shapes/quad.wgsl").into()),
            })
        }

        fn vertex_buffer(&self) -> &VertexBuffer<Self::Vertex> {
            &self.vertex_buffer
        }

        fn index_buffer(&self) -> &IndexBuffer<Self::Index> {
            &self.index_buffer
        }

        fn model_view(&self) -> Option<&UniformBuffer<[[f32; 4]; 4]>> {
            Some(&self.model_view)
        }
    }

    impl Quad {
        const VERTICES: [Vertex2d; 4] = [
            Vertex2d::new([0.0, 0.0]),
            Vertex2d::new([1.0, 0.0]),
            Vertex2d::new([0.0, 1.0]),
            Vertex2d::new([1.0, 1.0]),
        ];

        const INDICES: [u16; 6] = [0, 1, 3, 0, 2, 3];

        pub fn create(context: &Context) -> Self {
            Self {
                vertex_buffer: VertexBuffer::create_init(context.wgpu_device(), &Self::VERTICES),
                index_buffer: IndexBuffer::create_init(context.wgpu_device(), &Self::INDICES),
                model_view: UniformBuffer::create_init(
                    context.wgpu_device(),
                    Matrix4::identity().into(),
                ),
                uv_transform: UniformBuffer::create_init(
                    context.wgpu_device(),
                    Matrix4::identity().into(),
                ),
            }
        }
    }

    /// A generic 3d mesh without UV.
    #[derive(Debug, Clone)]
    pub struct Mesh3D {
        vertex_buffer: VertexBuffer<Vertex3dUV>,
        index_buffer: IndexBuffer<u32>,
        model_view: UniformBuffer<[[f32; 4]; 4]>,
    }

    impl_as_bind_group! {
        Mesh3D {
            0 => model_view,
        }
    }

    impl Mesh3D {
        pub fn create(context: &Context, vertices: &[Vertex3dUV], indices: &[u32]) -> Self {
            Self {
                vertex_buffer: VertexBuffer::create_init(context.wgpu_device(), vertices),
                index_buffer: IndexBuffer::create_init(context.wgpu_device(), indices),
                model_view: UniformBuffer::create_init(
                    context.wgpu_device(),
                    Matrix4::identity().into(),
                ),
            }
        }
    }

    impl AsMesh for Mesh3D {
        type Vertex = Vertex3dUV;

        type Index = u32;

        fn create_vertex_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/shapes/3d.wgsl").into()),
            })
        }

        fn vertex_buffer(&self) -> &VertexBuffer<Self::Vertex> {
            &self.vertex_buffer
        }

        fn index_buffer(&self) -> &IndexBuffer<Self::Index> {
            &self.index_buffer
        }

        fn model_view(&self) -> Option<&UniformBuffer<[[f32; 4]; 4]>> {
            Some(&self.model_view)
        }
    }
}

#[derive(Clone)]
pub(crate) struct MeshStorage {
    pub(crate) instance: Arc<dyn DynMesh>,
    pub(crate) vertex_shader: wgpu::ShaderModule,
    pub(crate) wgpu_bind_group: wgpu::BindGroup,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    pub(crate) index_format: wgpu::IndexFormat,
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
    pub(crate) fn new<Mesh: AsMesh>(device: &wgpu::Device, mesh_instance: Arc<Mesh>) -> Self {
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

    pub(crate) fn vertex_buffer(&self) -> &wgpu::Buffer {
        self.instance.vertex_buffer()
    }

    pub(crate) fn index_buffer(&self) -> &wgpu::Buffer {
        self.instance.index_buffer()
    }

    pub(crate) fn index_buffer_length(&self) -> u32 {
        self.instance.index_buffer_length()
    }
}
