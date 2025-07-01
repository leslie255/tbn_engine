use std::marker::PhantomData;

use bytemuck::Pod;
use cgmath::*;
use wgpu::{ShaderStages, TextureUsages, util::DeviceExt as _};

pub trait Bindable {
    fn bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry;
    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry;
}

pub trait AsBindGroup {
    fn bind_group_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry>;
    fn bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry>;
}

/// TODO: Make this into a derive macro so it supports structs with generic parameters.
#[macro_export]
macro_rules! impl_as_bind_group {
    ($T:path { $($binding_id:literal => $field:ident),* $(,)? } $($tts:tt)*) => {
        impl $crate::binding::AsBindGroup for $T {
            fn bind_group_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
                std::vec![
                    $($crate::binding::Bindable::bind_group_layout_entry(
                        &self.$field,
                        $binding_id,
                    )),*
                ]
            }
            fn bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
                std::vec![
                    $($crate::binding::Bindable::bind_group_entry(
                        &self.$field,
                        $binding_id,
                    )),*
                ]
            }
        }
        impl_as_bind_group! { $($tts)* }
    };
    () => {}
}

pub(crate) fn create_wgpu_bind_group_layout(
    device: &wgpu::Device,
    bind_group: &impl AsBindGroup,
) -> wgpu::BindGroupLayout {
    let label = Some(std::any::type_name_of_val(&bind_group));
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label,
        entries: &bind_group.bind_group_layout_entries(),
    })
}

pub(crate) fn create_wgpu_bind_group(
    device: &wgpu::Device,
    bind_group: &impl AsBindGroup,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let label = Some(std::any::type_name_of_val(&bind_group));
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label,
        entries: &bind_group.bind_group_layout_entries(),
    });
    let wgpu_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout: &layout,
        entries: &bind_group.bind_group_entries(),
    });
    (wgpu_bind_group, layout)
}

#[derive(Debug, Clone)]
pub struct UniformBuffer<T: Pod + Copy> {
    wgpu_buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

impl<T: Pod + Copy> UniformBuffer<T> {
    pub fn create_init(device: &wgpu::Device, contents: T) -> Self {
        let bytes = bytemuck::bytes_of(&contents);
        let wgpu_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            wgpu_buffer,
            _marker: PhantomData,
        }
    }

    pub fn wgpu_buffer(&self) -> &wgpu::Buffer {
        &self.wgpu_buffer
    }

    pub fn wgpu_buffer_mut(&mut self) -> &mut wgpu::Buffer {
        &mut self.wgpu_buffer
    }

    pub fn write(&self, contents: T, queue: &wgpu::Queue) {
        queue.write_buffer(self.wgpu_buffer(), 0, bytemuck::bytes_of(&contents));
    }
}

impl<T: Pod + Copy> Bindable for UniformBuffer<T> {
    fn bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: self.wgpu_buffer().as_entire_binding(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture2d {
    wgpu_texture: wgpu::Texture,
    format: wgpu::TextureFormat,
    size: Vector2<u32>,
    usage: wgpu::TextureUsages,
}

impl Texture2d {
    fn extend(size: Vector2<u32>) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        }
    }

    pub fn create(
        device: &wgpu::Device,
        size: Vector2<u32>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: Self::extend(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        Self {
            wgpu_texture,
            format,
            size,
            usage,
        }
    }

    /// Creates a 2D texture of usage (COPY_DST | TEXTURE_BINDING) and then initialize it with data.
    pub fn create_init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: Vector2<u32>,
        format: wgpu::TextureFormat,
        data: &[u8],
    ) -> Self {
        let usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
        let descriptor = wgpu::TextureDescriptor {
            label: None,
            size: Self::extend(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        };
        let wgpu_texture =
            device.create_texture_with_data(queue, &descriptor, Default::default(), data);
        Self {
            wgpu_texture,
            format,
            size,
            usage,
        }
    }

    pub fn wgpu_texture(&self) -> &wgpu::Texture {
        &self.wgpu_texture
    }

    pub fn view(&self, sample_type: wgpu::TextureSampleType) -> TextureView2d {
        let wgpu_texture_view = self
            .wgpu_texture()
            .create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(self.format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                usage: Some(self.usage),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
        TextureView2d {
            wgpu_texture_view,
            format: self.format,
            size: self.size,
            sample_type,
        }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn size(&self) -> Vector2<u32> {
        self.size
    }

    pub fn usage(&self) -> wgpu::TextureUsages {
        self.usage
    }
}

#[derive(Debug, Clone)]
pub struct TextureView2d {
    wgpu_texture_view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    size: Vector2<u32>,
    sample_type: wgpu::TextureSampleType,
}

impl TextureView2d {
    pub(crate) fn from_raw(
        wgpu_texture_view: wgpu::TextureView,
        format: wgpu::TextureFormat,
        size: Vector2<u32>,
        sample_type: wgpu::TextureSampleType,
    ) -> Self {
        Self {
            wgpu_texture_view,
            format,
            size,
            sample_type,
        }
    }

    pub fn wgpu_texture_view(&self) -> &wgpu::TextureView {
        &self.wgpu_texture_view
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn size(&self) -> Vector2<u32> {
        self.size
    }
}

impl Bindable for TextureView2d {
    fn bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Texture {
                sample_type: self.sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&self.wgpu_texture_view),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sampler2d {
    wgpu_sampler: wgpu::Sampler,
}

impl Sampler2d {
    pub fn create(
        device: &wgpu::Device,
        address_mode: wgpu::AddressMode,
        mag_filter: wgpu::FilterMode,
        min_filter: wgpu::FilterMode,
    ) -> Self {
        let wgpu_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter,
            min_filter,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        Self { wgpu_sampler }
    }

    pub fn wgpu_sampler(&self) -> &wgpu::Sampler {
        &self.wgpu_sampler
    }
}

impl Bindable for Sampler2d {
    fn bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::all(),
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(self.wgpu_sampler()),
        }
    }
}
