use cgmath::*;

use crate::{binding::{self, UniformBuffer}, impl_as_bind_group};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraDirection {
    LookAt(Point3<f32>),
    LookTo(Vector3<f32>),
}

/// A 2D orthographical camera.
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Point3<f32>,
    pub up: Vector3<f32>,
    pub direction: CameraDirection,
    /// FOV = 0 for orthographical projection.
    /// This includes both +0.0f32 and -0.0f32.
    pub fov: Rad<f32>,
    pub near: f32,
    pub far: f32,
    projection: UniformBuffer<[[f32; 4]; 4]>,
    wgpu_bind_group: Option<wgpu::BindGroup>,
    wgpu_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl_as_bind_group! {
    Camera {
        0 => projection,
    }
}

impl Camera {
    pub fn create(
        device: &wgpu::Device,
        position: Point3<f32>,
        up: Vector3<f32>,
        direction: CameraDirection,
        fov: impl Into<Rad<f32>>,
        near: f32,
        far: f32,
    ) -> Self {
        let mut self_ = Self {
            position,
            up,
            direction,
            fov: fov.into(),
            near,
            far,
            projection: UniformBuffer::create_init(device, Matrix4::identity().into()),
            wgpu_bind_group: None,
            wgpu_bind_group_layout: None,
        };
        let (wgpu_bind_group, wgpu_bind_group_layout) =
            binding::create_wgpu_bind_group(device, &self_);
        self_.wgpu_bind_group = Some(wgpu_bind_group);
        self_.wgpu_bind_group_layout = Some(wgpu_bind_group_layout);
        self_
    }

    pub fn wgpu_bind_group(&self) -> &wgpu::BindGroup {
        self.wgpu_bind_group.as_ref().unwrap()
    }

    pub fn update_projection_uniform(&self, canvas_size: Vector2<f32>, queue: &wgpu::Queue) {
        self.projection
            .write(self.projection_matrix(canvas_size).into(), queue);
    }

    /// Use this camera for subsequent draw calls in this render pass.
    pub fn use_(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(0, self.wgpu_bind_group(), &[]);
    }

    pub fn projection_matrix(&self, canvas_size: Vector2<f32>) -> Matrix4<f32> {
        if self.fov.is_zero() {
            cgmath::ortho(
                -canvas_size.x / 2.0,
                canvas_size.x / 2.0,
                -canvas_size.y / 2.0,
                canvas_size.y / 2.0,
                -1.0,
                1.0,
            )
        } else {
            cgmath::perspective(self.fov, canvas_size.x / canvas_size.y, self.near, self.far)
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        match self.direction {
            CameraDirection::LookAt(target) => Matrix4::look_at_rh(self.position, target, self.up),
            CameraDirection::LookTo(direction) => {
                Matrix4::look_to_rh(self.position, direction, self.up)
            }
        }
    }
}
