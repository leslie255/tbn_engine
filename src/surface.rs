use std::sync::Arc;

use cgmath::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    DepthStencilTexture2d, DepthStencilTextureFormat, DepthStencilTextureView2d, Texture2d,
    TextureFormat, TextureView2d, TextureView2d_,
};

#[derive(Debug)]
pub struct WindowSurface {
    format: TextureFormat,
    wgpu_surface: wgpu::Surface<'static>,
    depth_stencil_texture: DepthStencilTexture2d,
    physical_size: Vector2<u32>,
    window: Arc<Window>,
}

impl WindowSurface {
    pub fn new(
        window: Arc<Window>,
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
    ) -> Self {
        let wgpu_surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let capabilities = wgpu_surface.get_capabilities(adapter);
        let format = capabilities.formats[0];
        let size = window.inner_size();
        let size = vec2(size.width, size.height);
        let self_ = Self {
            format: format.try_into().unwrap(),
            wgpu_surface,
            depth_stencil_texture: Self::create_depth_stencil_texture(device, size),
            physical_size: size,
            window,
        };
        self_.configure_surface(device);
        self_
    }

    fn create_depth_stencil_texture(
        device: &wgpu::Device,
        size: Vector2<u32>,
    ) -> DepthStencilTexture2d {
        DepthStencilTexture2d::create(
            device,
            vec2(size.x, size.y),
            DepthStencilTextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        )
    }

    fn configure_surface(&self, device: &wgpu::Device) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.format().into(),
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.format().to_wgpu_texture_format().add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.physical_size().x,
            height: self.physical_size().y,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.wgpu_surface.configure(device, &surface_config);
    }

    pub fn frame(&self, f: impl FnOnce(SurfaceView)) {
        let surface_texture = self.wgpu_surface.get_current_texture().unwrap();
        let wgpu_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.format().into()),
                ..Default::default()
            });
        let texture_view = TextureView2d_::from_raw(
            wgpu_texture_view,
            self.format,
            self.physical_size,
            wgpu::TextureSampleType::default(),
        );
        let surface = SurfaceView::new(
            texture_view,
            self.depth_stencil_texture.view(Default::default()),
        );
        f(surface);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    pub fn resized(&mut self, new_size: PhysicalSize<u32>, device: &wgpu::Device) {
        self.physical_size = vec2(new_size.width, new_size.height);
        self.configure_surface(device);
        self.depth_stencil_texture = Self::create_depth_stencil_texture(device, self.physical_size);
    }

    pub fn physical_size(&self) -> Vector2<u32> {
        self.physical_size
    }

    pub fn physical_size_f32(&self) -> Vector2<f32> {
        self.physical_size.map(|u| u as f32)
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn depth_stencil_format(&self) -> DepthStencilTextureFormat {
        self.depth_stencil_texture.format()
    }
}

/// A target for drawing.
#[derive(Debug, Clone)]
pub struct Surface {
    format: TextureFormat,
    color_texture: Texture2d,
    depth_stencil_texture: DepthStencilTexture2d,
}

impl Surface {
    pub fn create(device: &wgpu::Device, size: Vector2<u32>, format: TextureFormat) -> Self {
        Self {
            format,
            color_texture: Texture2d::create(
                device,
                size,
                format,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            depth_stencil_texture: DepthStencilTexture2d::create(
                device,
                size,
                DepthStencilTextureFormat::Depth32Float,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
        }
    }

    pub fn color_texture(&self) -> &Texture2d {
        &self.color_texture
    }

    pub fn depth_stencil_texture(&self) -> &DepthStencilTexture2d {
        &self.depth_stencil_texture
    }

    pub fn view(&self) -> SurfaceView {
        SurfaceView {
            color_texture: self.color_texture().view(Default::default()),
            depth_stencil_texture: self
                .depth_stencil_texture
                .view(wgpu::TextureSampleType::Depth),
        }
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn size(&self) -> Vector2<u32> {
        self.color_texture().size()
    }

    pub fn size_f32(&self) -> Vector2<f32> {
        self.size().map(|u| u as f32)
    }
}

/// View of a surface.
#[derive(Debug, Clone)]
pub struct SurfaceView {
    color_texture: TextureView2d,
    depth_stencil_texture: DepthStencilTextureView2d,
}

impl SurfaceView {
    pub fn new(texture: TextureView2d, depth_stencil_texture: DepthStencilTextureView2d) -> Self {
        Self {
            color_texture: texture,
            depth_stencil_texture,
        }
    }

    pub fn render_pass_with_descriptor(
        &self,
        device: &wgpu::Device,
        descriptor: &wgpu::RenderPassDescriptor,
    ) -> RenderPass {
        let mut encoder = device.create_command_encoder(&Default::default());
        let render_pass = encoder.begin_render_pass(descriptor).forget_lifetime();
        RenderPass {
            wgpu_encoder: encoder,
            wgpu_render_pass: render_pass,
        }
    }

    pub fn render_pass(&self, device: &wgpu::Device) -> RenderPass {
        self.render_pass_with_descriptor(device, &wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.color_texture.wgpu_texture_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.depth_stencil_texture.wgpu_texture_view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    pub fn format(&self) -> TextureFormat {
        self.color_texture().format()
    }

    pub fn depth_stencil_format(&self) -> DepthStencilTextureFormat {
        self.depth_stencil_texture().format()
    }

    pub fn size(&self) -> Vector2<u32> {
        self.color_texture.size()
    }

    pub fn size_f32(&self) -> Vector2<f32> {
        self.color_texture.size().map(|u| u as f32)
    }

    pub fn color_texture(&self) -> &TextureView2d {
        &self.color_texture
    }

    pub fn depth_stencil_texture(&self) -> &DepthStencilTextureView2d {
        &self.depth_stencil_texture
    }

    /// Returns the color texture view (`.0`) and the depth stencil texture view (`.1`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn f(surface_view: SurfaceView) {
    /// let (color_texture, depth_stencil_texture) =
    ///     surface_view.into_color_depth_stencil_textures();
    /// }
    /// ```
    pub fn into_color_depth_stencil_textures(self) -> (TextureView2d, DepthStencilTextureView2d) {
        (self.color_texture, self.depth_stencil_texture)
    }
}

#[derive(Debug)]
pub struct RenderPass {
    wgpu_encoder: wgpu::CommandEncoder,
    wgpu_render_pass: wgpu::RenderPass<'static>,
}

impl RenderPass {
    pub fn finish(self, queue: &wgpu::Queue) {
        drop(self.wgpu_render_pass);
        queue.submit([self.wgpu_encoder.finish()]);
    }

    pub fn wgpu_render_pass(&self) -> &wgpu::RenderPass<'static> {
        &self.wgpu_render_pass
    }

    pub fn wgpu_render_pass_mut(&mut self) -> &mut wgpu::RenderPass<'static> {
        &mut self.wgpu_render_pass
    }

    pub fn set_bind_group<'a, BG>(
        &mut self,
        index: u32,
        bind_group: BG,
        offsets: &[wgpu::DynamicOffset],
    ) where
        Option<&'a wgpu::BindGroup>: From<BG>,
    {
        self.wgpu_render_pass
            .set_bind_group(index, bind_group, offsets)
    }
}
