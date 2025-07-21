use std::{
    ops::DerefMut,
    sync::{
        Arc, Mutex,
        atomic::{self, AtomicU64},
    },
};

use crate::{AsMaterial, AsMesh, Camera, MaterialStorage, MeshStorage, ObjectStorage, Scene};

#[derive(Debug)]
pub struct Context {
    wgpu_device: wgpu::Device,
    wgpu_queue: wgpu::Queue,
    object_id_counter: AtomicU64,
}

impl Context {
    pub fn new(wgpu_device: wgpu::Device, wgpu_queue: wgpu::Queue) -> Self {
        Self {
            wgpu_device,
            wgpu_queue,
            object_id_counter: AtomicU64::new(0),
        }
    }

    pub fn wgpu_device(&self) -> &wgpu::Device {
        &self.wgpu_device
    }

    pub fn wgpu_queue(&self) -> &wgpu::Queue {
        &self.wgpu_queue
    }

    fn increment_object_id_counter(&self) -> u64 {
        self.object_id_counter
            .fetch_add(1, atomic::Ordering::Relaxed)
    }

    pub fn create_mesh(&self, mesh_instance: Arc<impl AsMesh>) -> MeshRef {
        let mesh_storage = MeshStorage::new(self.wgpu_device(), mesh_instance);
        MeshRef::new(mesh_storage)
    }

    pub fn create_material(&self, material_instance: &impl AsMaterial) -> MaterialRef {
        let material_storage = MaterialStorage::new(self.wgpu_device(), material_instance);
        MaterialRef::new(material_storage)
    }

    pub fn create_camera(&self, camera_instance: Camera) -> CameraRef {
        CameraRef::new(camera_instance)
    }

    pub fn create_object(
        &self,
        scene: &Scene,
        camera: CameraRef,
        mesh: MeshRef,
        material: MaterialRef,
    ) -> ObjectRef {
        let id = self.increment_object_id_counter();
        let object_storage =
            ObjectStorage::new(scene, id, self.wgpu_device(), camera, mesh, material);
        ObjectRef::new(object_storage)
    }
}

// TODO: Perhaps use a third-party, `Weak`-less `Arc`.
macro_rules! define_ref_type {
    ($T:ident, $Storage:ty $(,)?) => {
        #[allow(dead_code)]
        #[derive(Debug, Clone)]
        pub struct $T {
            storage: Arc<Mutex<$Storage>>,
        }
        impl $T {
            pub(crate) fn new(storage: $Storage) -> Self {
                Self {
                    storage: Arc::new(Mutex::new(storage)),
                }
            }
            #[track_caller]
            pub(crate) fn lock(&self) -> impl DerefMut<Target = $Storage> {
                self.storage.lock().unwrap()
            }
        }
    };
}

define_ref_type!(MeshRef, MeshStorage);
define_ref_type!(MaterialRef, MaterialStorage);
define_ref_type!(ObjectRef, ObjectStorage);
define_ref_type!(CameraRef, Camera);

impl ObjectRef {
    pub fn set_is_hidden(&self, is_hidden: bool) {
        self.lock().is_hidden = is_hidden;
    }

    pub fn get_is_hidden(&self) -> bool {
        self.lock().is_hidden
    }
}

impl CameraRef {
    pub fn with_mut<T>(&self, f: impl FnOnce(&mut Camera) -> T) -> T {
        // Operates on a copy of the camera in case user tries to render while inside the closure.
        let mut camera_copy = self.lock().clone();
        let result = f(&mut camera_copy);
        *self.lock() = camera_copy;
        result
    }
}
