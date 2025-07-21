use crate::{binding, impl_as_bind_group, AsBindGroup, Rgba, Sampler, UniformBuffer};

pub trait AsMaterial: AsBindGroup {
    fn create_fragment_shader(device: &wgpu::Device) -> wgpu::ShaderModule;

    fn blend_state() -> Option<wgpu::BlendState> {
        Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                operation: wgpu::BlendOperation::Add,
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            },
            alpha: wgpu::BlendComponent::REPLACE,
        })
    }
}

pub mod materials {
    use crate::{Context, TextureView2d};

    use super::*;

    #[derive(Debug, Clone)]
    pub struct UniformFill {
        pub fill_color: UniformBuffer<Rgba>,
    }

    impl_as_bind_group! {
        UniformFill {
            0 => fill_color,
        }
    }

    impl UniformFill {
        pub fn create(context: &Context, color: Rgba) -> Self {
            Self {
                fill_color: UniformBuffer::create_init(context.wgpu_device(), color),
            }
        }
    }

    impl AsMaterial for UniformFill {
        fn create_fragment_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/materials/uniform_fill.wgsl").into(),
                ),
            })
        }
    }

    #[derive(Debug, Clone)]
    pub struct SdfCircle {
        pub fill_color: UniformBuffer<Rgba>,
        /// The center of the circle, in UV space.
        pub center: UniformBuffer<[f32; 2]>,
        /// The radius of the circle, in UV space.
        pub radius: UniformBuffer<f32>,
    }

    impl_as_bind_group! {
        SdfCircle {
            0 => fill_color,
            1 => center,
            2 => radius,
        }
    }

    impl SdfCircle {
        pub fn create(context: &Context, fill_color: Rgba) -> Self {
            Self {
                fill_color: UniformBuffer::create_init(context.wgpu_device(), fill_color),
                center: UniformBuffer::create_init(context.wgpu_device(), [0.5; 2]),
                radius: UniformBuffer::create_init(context.wgpu_device(), 0.5),
            }
        }
    }

    impl AsMaterial for SdfCircle {
        fn create_fragment_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/materials/sdf_circle.wgsl").into(),
                ),
            })
        }
    }

    #[derive(Debug, Clone)]
    pub struct Textured {
        texture_view: TextureView2d,
        sampler: Sampler,
    }

    impl_as_bind_group! {
        Textured {
            0 => texture_view,
            1 => sampler,
        }
    }

    impl Textured {
        pub fn create(texture_view: TextureView2d, sampler: Sampler) -> Self {
            Self {
                texture_view,
                sampler,
            }
        }

        pub fn texture_view(&self) -> &TextureView2d {
            &self.texture_view
        }

        pub fn sampler(&self) -> &Sampler {
            &self.sampler
        }
    }

    impl AsMaterial for Textured {
        fn create_fragment_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/materials/textured.wgsl").into(),
                ),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MaterialStorage {
    pub(crate) fragment_shader: wgpu::ShaderModule,
    pub(crate) wgpu_bind_group: wgpu::BindGroup,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) blend_state: Option<wgpu::BlendState>,
}

impl MaterialStorage {
    pub(crate) fn new<Material: AsMaterial>(
        device: &wgpu::Device,
        material_instance: &Material,
    ) -> Self {
        let (wgpu_bind_group, bind_group_layout) =
            binding::create_wgpu_bind_group(device, material_instance);
        Self {
            fragment_shader: Material::create_fragment_shader(device),
            wgpu_bind_group,
            bind_group_layout,
            blend_state: Material::blend_state(),
        }
    }
}

