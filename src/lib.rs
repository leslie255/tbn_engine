/// Contains the `Bindable` and `AsBindGroup` traits, and functions for creating wgpu bind groups
/// and bind group layouts.
pub(crate) mod binding;
/// Contains vertex, index, and uniform buffers.
pub(crate) mod buffers;
/// Contains data structures for camera.
pub(crate) mod camera;
/// Contains data structures for colors.
pub(crate) mod color;
/// Contains the `AsMaterial` trait and various materials.
pub(crate) mod material;
/// Contains the `AsMesh` trait and various meshes.
pub(crate) mod mesh;
/// Contains `Scene`, various ID types, and data structures used internally in `Scene`.
pub(crate) mod scene;
/// Contains `Surface`, `SurfaceView`, `WindowSurface`, and `RenderPass`.
pub(crate) mod surface;
/// Contains textures, texture views, texture formats, and samplers.
pub(crate) mod texture;

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
