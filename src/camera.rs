use cgmath::*;

use crate::{UniformBuffer, impl_as_bind_group};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraDirection {
    LookAt(Point3<f32>),
    LookTo(Vector3<f32>),
}

/// A 2D orthographical camera.
#[derive(Debug, Clone)]
pub struct Camera {
    // Camera isn't `Copy` for stability sake.
    pub position: Point3<f32>,
    pub up: Vector3<f32>,
    pub direction: CameraDirection,
    /// FOV = 0 for orthographical projection.
    /// This includes both +0.0f32 and -0.0f32.
    pub fov: Rad<f32>,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        up: Vector3<f32>,
        direction: CameraDirection,
        fov: impl Into<Rad<f32>>,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            up,
            direction,
            fov: fov.into(),
            near,
            far,
        }
    }

    pub fn projection_matrix(&self, viewport_size: Vector2<f32>) -> Matrix4<f32> {
        if self.fov.0.is_zero() {
            cgmath::ortho(
                -viewport_size.x / 2.0,
                viewport_size.x / 2.0,
                -viewport_size.y / 2.0,
                viewport_size.y / 2.0,
                self.near,
                self.far,
            )
        } else {
            cgmath::perspective(
                self.fov,
                viewport_size.x / viewport_size.y,
                self.near,
                self.far,
            )
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

#[derive(Debug, Clone)]
pub struct CameraBindGroup {
    pub projection: UniformBuffer<[[f32; 4]; 4]>,
}

impl CameraBindGroup {
    pub fn create(device: &wgpu::Device) -> Self {
        Self {
            projection: UniformBuffer::create_init(device, Matrix4::identity().into()),
        }
    }
}

impl_as_bind_group! {
    CameraBindGroup {
        0 => projection,
    }
}
