use cgmath::*;
use wgpu::{ShaderStages, TextureUsages, util::DeviceExt as _};

use crate::{Bindable, Context};

/// A trait that `TextureFormat`, `DepthStencilTextureFormat`, and `wgpu::TextureFormat` all implements.
/// This is for easier code reuse for texture and texture view structs.
pub trait TextureFormatTrait:
    TryFrom<wgpu::TextureFormat> + Into<wgpu::TextureFormat> + Copy + Eq
{
    const SUPPORTS_DEPTH_STENCIL_VIEW: bool = false;
    const IS_NORMALIZED: bool = false;
}

impl TextureFormatTrait for wgpu::TextureFormat {}

/// Uncompressed, normalized, color texture formats.
/// Subset of `wgpu::TextureFormat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    // Normal 8 bit formats
    /// Red channel only. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    R8Unorm,
    /// Red channel only. 8 bit integer per channel. [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    R8Snorm,

    // Normal 16 bit formats
    /// Red channel only. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    R16Unorm,
    /// Red channel only. 16 bit integer per channel. [&minus;32767, 32767] converted to/from float [&minus;1, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    R16Snorm,
    /// Red channel only. 16 bit float per channel. Float in shader.
    R16Float,
    /// Red and green channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Rg8Unorm,
    /// Red and green channels. 8 bit integer per channel. [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    Rg8Snorm,

    // Normal 32 bit formats
    /// Red channel only. 32 bit integer per channel. Unsigned in shader.
    R32Uint,
    /// Red channel only. 32 bit float per channel. Float in shader.
    R32Float,
    /// Red and green channels. 16 bit integer per channel. Unsigned in shader.
    Rg16Uint,
    /// Red and green channels. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    Rg16Unorm,
    /// Red and green channels. 16 bit integer per channel. [&minus;32767, 32767] converted to/from float [&minus;1, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    Rg16Snorm,
    /// Red and green channels. 16 bit float per channel. Float in shader.
    Rg16Float,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Rgba8Unorm,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    Rgba8UnormSrgb,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    Rgba8Snorm,
    /// Red, green, blue, and alpha channels. 8 bit integer per channel. Unsigned in shader.
    Rgba8Uint,
    /// Blue, green, red, and alpha channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Bgra8Unorm,
    /// Blue, green, red, and alpha channels. 8 bit integer per channel. Srgb-color [0, 255] converted to/from linear-color float [0, 1] in shader.
    Bgra8UnormSrgb,

    // Packed 32 bit formats
    /// Packed unsigned float with 9 bits mantisa for each RGB component, then a common 5 bits exponent
    Rgb9e5Ufloat,
    /// Red, green, blue, and alpha channels. 10 bit integer for RGB channels, 2 bit integer for alpha channel. Unsigned in shader.
    Rgb10a2Uint,
    /// Red, green, blue, and alpha channels. 10 bit integer for RGB channels, 2 bit integer for alpha channel. [0, 1023] ([0, 3] for alpha) converted to/from float [0, 1] in shader.
    Rgb10a2Unorm,
    /// Red, green, and blue channels. 11 bit float with no sign bit for RG channels. 10 bit float with no sign bit for blue channel. Float in shader.
    Rg11b10Ufloat,

    // Normal 64 bit formats
    /// Red channel only. 64 bit integer per channel. Unsigned in shader.
    ///
    /// [`Features::TEXTURE_INT64_ATOMIC`] must be enabled to use this texture format.
    R64Uint,
    /// Red and green channels. 32 bit integer per channel. Unsigned in shader.
    Rg32Uint,
    /// Red and green channels. 32 bit float per channel. Float in shader.
    Rg32Float,
    /// Red, green, blue, and alpha channels. 16 bit integer per channel. Unsigned in shader.
    Rgba16Uint,
    /// Red, green, blue, and alpha channels. 16 bit integer per channel. [0, 65535] converted to/from float [0, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    Rgba16Unorm,
    /// Red, green, blue, and alpha. 16 bit integer per channel. [&minus;32767, 32767] converted to/from float [&minus;1, 1] in shader.
    ///
    /// [`Features::TEXTURE_FORMAT_16BIT_NORM`] must be enabled to use this texture format.
    Rgba16Snorm,
    /// Red, green, blue, and alpha channels. 16 bit float per channel. Float in shader.
    Rgba16Float,

    // Normal 128 bit formats
    /// Red, green, blue, and alpha channels. 32 bit integer per channel. Unsigned in shader.
    Rgba32Uint,
    /// Red, green, blue, and alpha channels. 32 bit float per channel. Float in shader.
    Rgba32Float,
}

impl From<TextureFormat> for wgpu::TextureFormat {
    fn from(value: TextureFormat) -> Self {
        value.to_wgpu_texture_format()
    }
}

#[derive(Debug, Clone)]
pub struct TextureFormatFromWgpuTextureFormatError {
    /// For stability sake.
    _private: (),
}

impl TryFrom<wgpu::TextureFormat> for TextureFormat {
    type Error = TextureFormatFromWgpuTextureFormatError;
    fn try_from(value: wgpu::TextureFormat) -> Result<Self, Self::Error> {
        Self::from_wgpu_texture_format(value)
            .ok_or(TextureFormatFromWgpuTextureFormatError { _private: () })
    }
}

impl TextureFormatTrait for TextureFormat {
    const IS_NORMALIZED: bool = true;
}

impl TextureFormat {
    pub const fn to_wgpu_texture_format(self) -> wgpu::TextureFormat {
        match self {
            TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
            TextureFormat::R8Snorm => wgpu::TextureFormat::R8Snorm,
            TextureFormat::R16Unorm => wgpu::TextureFormat::R16Unorm,
            TextureFormat::R16Snorm => wgpu::TextureFormat::R16Snorm,
            TextureFormat::R16Float => wgpu::TextureFormat::R16Float,
            TextureFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
            TextureFormat::Rg8Snorm => wgpu::TextureFormat::Rg8Snorm,
            TextureFormat::R32Uint => wgpu::TextureFormat::R32Uint,
            TextureFormat::R32Float => wgpu::TextureFormat::R32Float,
            TextureFormat::Rg16Uint => wgpu::TextureFormat::Rg16Uint,
            TextureFormat::Rg16Unorm => wgpu::TextureFormat::Rg16Unorm,
            TextureFormat::Rg16Snorm => wgpu::TextureFormat::Rg16Snorm,
            TextureFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
            TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
            TextureFormat::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
            TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Rgb9e5Ufloat => wgpu::TextureFormat::Rgb9e5Ufloat,
            TextureFormat::Rgb10a2Uint => wgpu::TextureFormat::Rgb10a2Uint,
            TextureFormat::Rgb10a2Unorm => wgpu::TextureFormat::Rgb10a2Unorm,
            TextureFormat::Rg11b10Ufloat => wgpu::TextureFormat::Rg11b10Ufloat,
            TextureFormat::R64Uint => wgpu::TextureFormat::R64Uint,
            TextureFormat::Rg32Uint => wgpu::TextureFormat::Rg32Uint,
            TextureFormat::Rg32Float => wgpu::TextureFormat::Rg32Float,
            TextureFormat::Rgba16Uint => wgpu::TextureFormat::Rgba16Uint,
            TextureFormat::Rgba16Unorm => wgpu::TextureFormat::Rgba16Unorm,
            TextureFormat::Rgba16Snorm => wgpu::TextureFormat::Rgba16Snorm,
            TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            TextureFormat::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
            TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        }
    }

    pub const fn from_wgpu_texture_format(wgpu_format: wgpu::TextureFormat) -> Option<Self> {
        match wgpu_format {
            wgpu::TextureFormat::R8Unorm => Some(Self::R8Unorm),
            wgpu::TextureFormat::R8Snorm => Some(Self::R8Snorm),
            wgpu::TextureFormat::R16Unorm => Some(Self::R16Unorm),
            wgpu::TextureFormat::R16Snorm => Some(Self::R16Snorm),
            wgpu::TextureFormat::R16Float => Some(Self::R16Float),
            wgpu::TextureFormat::Rg8Unorm => Some(Self::Rg8Unorm),
            wgpu::TextureFormat::Rg8Snorm => Some(Self::Rg8Snorm),
            wgpu::TextureFormat::R32Uint => Some(Self::R32Uint),
            wgpu::TextureFormat::R32Float => Some(Self::R32Float),
            wgpu::TextureFormat::Rg16Uint => Some(Self::Rg16Uint),
            wgpu::TextureFormat::Rg16Unorm => Some(Self::Rg16Unorm),
            wgpu::TextureFormat::Rg16Snorm => Some(Self::Rg16Snorm),
            wgpu::TextureFormat::Rg16Float => Some(Self::Rg16Float),
            wgpu::TextureFormat::Rgba8Unorm => Some(Self::Rgba8Unorm),
            wgpu::TextureFormat::Rgba8UnormSrgb => Some(Self::Rgba8UnormSrgb),
            wgpu::TextureFormat::Rgba8Snorm => Some(Self::Rgba8Snorm),
            wgpu::TextureFormat::Rgba8Uint => Some(Self::Rgba8Uint),
            wgpu::TextureFormat::Bgra8Unorm => Some(Self::Bgra8Unorm),
            wgpu::TextureFormat::Bgra8UnormSrgb => Some(Self::Bgra8UnormSrgb),
            wgpu::TextureFormat::Rgb9e5Ufloat => Some(Self::Rgb9e5Ufloat),
            wgpu::TextureFormat::Rgb10a2Uint => Some(Self::Rgb10a2Uint),
            wgpu::TextureFormat::Rgb10a2Unorm => Some(Self::Rgb10a2Unorm),
            wgpu::TextureFormat::Rg11b10Ufloat => Some(Self::Rg11b10Ufloat),
            wgpu::TextureFormat::R64Uint => Some(Self::R64Uint),
            wgpu::TextureFormat::Rg32Uint => Some(Self::Rg32Uint),
            wgpu::TextureFormat::Rg32Float => Some(Self::Rg32Float),
            wgpu::TextureFormat::Rgba16Uint => Some(Self::Rgba16Uint),
            wgpu::TextureFormat::Rgba16Unorm => Some(Self::Rgba16Unorm),
            wgpu::TextureFormat::Rgba16Snorm => Some(Self::Rgba16Snorm),
            wgpu::TextureFormat::Rgba16Float => Some(Self::Rgba16Float),
            wgpu::TextureFormat::Rgba32Uint => Some(Self::Rgba32Uint),
            wgpu::TextureFormat::Rgba32Float => Some(Self::Rgba32Float),
            _ => None,
        }
    }
}

/// Depth-stencil texture formats.
/// Subset of `wgpu::TextureFormat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthStencilTextureFormat {
    /// Stencil format with 8 bit integer stencil.
    Stencil8,
    /// Special depth format with 16 bit integer depth.
    Depth16Unorm,
    /// Special depth format with at least 24 bit integer depth.
    Depth24Plus,
    /// Special depth/stencil format with at least 24 bit integer depth and 8 bits integer stencil.
    Depth24PlusStencil8,
    /// Special depth format with 32 bit floating point depth.
    Depth32Float,
    /// Special depth/stencil format with 32 bit floating point depth and 8 bits integer stencil.
    ///
    /// [`Features::DEPTH32FLOAT_STENCIL8`] must be enabled to use this texture format.
    Depth32FloatStencil8,
}

impl From<DepthStencilTextureFormat> for wgpu::TextureFormat {
    fn from(value: DepthStencilTextureFormat) -> Self {
        value.to_wgpu_texture_format()
    }
}

#[derive(Debug, Clone)]
pub struct DepthStencilTextureFormatFromWgpuTextureFormatError {
    /// For stability sake.
    _private: (),
}

impl TryFrom<wgpu::TextureFormat> for DepthStencilTextureFormat {
    type Error = TextureFormatFromWgpuTextureFormatError;
    fn try_from(value: wgpu::TextureFormat) -> Result<Self, Self::Error> {
        Self::from_wgpu_texture_format(value)
            .ok_or(TextureFormatFromWgpuTextureFormatError { _private: () })
    }
}

impl TextureFormatTrait for DepthStencilTextureFormat {
    const SUPPORTS_DEPTH_STENCIL_VIEW: bool = true;
}

impl DepthStencilTextureFormat {
    pub const fn to_wgpu_texture_format(self) -> wgpu::TextureFormat {
        match self {
            DepthStencilTextureFormat::Stencil8 => wgpu::TextureFormat::Stencil8,
            DepthStencilTextureFormat::Depth16Unorm => wgpu::TextureFormat::Depth16Unorm,
            DepthStencilTextureFormat::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
            DepthStencilTextureFormat::Depth24PlusStencil8 => {
                wgpu::TextureFormat::Depth24PlusStencil8
            }
            DepthStencilTextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
            DepthStencilTextureFormat::Depth32FloatStencil8 => {
                wgpu::TextureFormat::Depth32FloatStencil8
            }
        }
    }

    pub const fn from_wgpu_texture_format(wgpu_format: wgpu::TextureFormat) -> Option<Self> {
        match wgpu_format {
            wgpu::TextureFormat::Stencil8 => Some(Self::Stencil8),
            wgpu::TextureFormat::Depth16Unorm => Some(Self::Depth16Unorm),
            wgpu::TextureFormat::Depth24Plus => Some(Self::Depth24Plus),
            wgpu::TextureFormat::Depth24PlusStencil8 => Some(Self::Depth24PlusStencil8),
            wgpu::TextureFormat::Depth32Float => Some(Self::Depth32Float),
            wgpu::TextureFormat::Depth32FloatStencil8 => Some(Self::Depth32FloatStencil8),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture2d_<Format: TextureFormatTrait> {
    wgpu_texture: wgpu::Texture,
    format: Format,
    size: Vector2<u32>,
    usage: wgpu::TextureUsages,
}

pub type Texture2d = Texture2d_<TextureFormat>;
pub type DepthStencilTexture2d = Texture2d_<DepthStencilTextureFormat>;
pub type GenericTexture2d = Texture2d_<wgpu::TextureFormat>;

impl<Format: TextureFormatTrait> Texture2d_<Format> {
    fn extent(size: Vector2<u32>) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        }
    }

    pub fn create(
        device: &wgpu::Device,
        size: Vector2<u32>,
        format: Format,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: Self::extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.into(),
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
    pub fn create_init(context: &Context, size: Vector2<u32>, format: Format, data: &[u8]) -> Self {
        let usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
        let descriptor = wgpu::TextureDescriptor {
            label: None,
            size: Self::extent(size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.into(),
            usage,
            view_formats: &[],
        };
        let wgpu_texture = context.wgpu_device().create_texture_with_data(
            context.wgpu_queue(),
            &descriptor,
            Default::default(),
            data,
        );
        Self {
            wgpu_texture,
            format,
            size,
            usage,
        }
    }

    pub fn wgpu_format(&self) -> wgpu::TextureFormat {
        self.format().into()
    }

    pub fn wgpu_texture(&self) -> &wgpu::Texture {
        &self.wgpu_texture
    }

    pub fn view(&self, sample_type: wgpu::TextureSampleType) -> TextureView2d_<Format> {
        let wgpu_texture_view = self
            .wgpu_texture()
            .create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(self.wgpu_format()),
                dimension: Some(wgpu::TextureViewDimension::D2),
                usage: Some(self.usage),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
        TextureView2d_ {
            wgpu_texture_view,
            format: self.format,
            size: self.size,
            sample_type,
        }
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn size(&self) -> Vector2<u32> {
        self.size
    }

    pub fn usage(&self) -> wgpu::TextureUsages {
        self.usage
    }

    pub fn into_generic_texture(self) -> GenericTexture2d {
        GenericTexture2d {
            wgpu_texture: self.wgpu_texture,
            format: self.format.into(),
            size: self.size,
            usage: self.usage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureView2d_<Format: TextureFormatTrait> {
    wgpu_texture_view: wgpu::TextureView,
    format: Format,
    size: Vector2<u32>,
    sample_type: wgpu::TextureSampleType,
}

pub type TextureView2d = TextureView2d_<TextureFormat>;
pub type DepthStencilTextureView2d = TextureView2d_<DepthStencilTextureFormat>;
pub type GenericTextureView2d = TextureView2d_<wgpu::TextureFormat>;

impl<Format: TextureFormatTrait> TextureView2d_<Format> {
    pub(crate) fn from_raw(
        wgpu_texture_view: wgpu::TextureView,
        format: Format,
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

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn wgpu_format(&self) -> wgpu::TextureFormat {
        self.format().into()
    }

    pub fn size(&self) -> Vector2<u32> {
        self.size
    }

    pub fn into_generic_texture_view(self) -> GenericTextureView2d {
        GenericTextureView2d {
            wgpu_texture_view: self.wgpu_texture_view,
            format: self.format.into(),
            size: self.size,
            sample_type: self.sample_type,
        }
    }
}

impl<Format: TextureFormatTrait> Bindable for TextureView2d_<Format> {
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
pub struct Sampler {
    wgpu_sampler: wgpu::Sampler,
}

impl Sampler {
    pub fn create(
        context: &Context,
        address_mode: wgpu::AddressMode,
        mag_filter: wgpu::FilterMode,
        min_filter: wgpu::FilterMode,
    ) -> Self {
        let wgpu_sampler = context.wgpu_device().create_sampler(&wgpu::SamplerDescriptor {
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

impl Bindable for Sampler {
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

#[derive(Debug, Clone)]
pub struct ComparingSampler {
    wgpu_sampler: wgpu::Sampler,
}

impl ComparingSampler {
    pub fn create(
        device: &wgpu::Device,
        address_mode: wgpu::AddressMode,
        mag_filter: wgpu::FilterMode,
        min_filter: wgpu::FilterMode,
        compare: wgpu::CompareFunction,
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
            compare: Some(compare),
            anisotropy_clamp: 1,
            border_color: None,
        });
        Self { wgpu_sampler }
    }

    pub fn wgpu_sampler(&self) -> &wgpu::Sampler {
        &self.wgpu_sampler
    }
}

impl Bindable for ComparingSampler {
    fn bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::all(),
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
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
