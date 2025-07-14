/// Contains the `Bindable` and `AsBindGroup` traits, and functions for creating wgpu bind groups
/// and bind group layouts.
mod binding;
/// Contains vertex, index, and uniform buffers.
mod buffers;
/// Contains data structures for camera.
mod camera;
/// Contains data structures for colors.
mod color;
/// Contains the `AsMaterial` trait and various materials.
mod material;
/// Contains the `AsMesh` trait and various meshes.
mod mesh;
/// Contains `Scene`, various ID types, and data structures used internally in `Scene`.
mod scene;
/// Contains `Surface`, `SurfaceView`, `WindowSurface`, and `RenderPass`.
mod surface;
/// Contains textures, texture views, texture formats, and samplers.
mod texture;

pub use binding::*;
pub use buffers::*;
pub use camera::*;
pub use color::*;
pub use material::*;
pub use mesh::*;
pub use scene::*;
pub use surface::*;
pub use texture::*;

pub use obj;
