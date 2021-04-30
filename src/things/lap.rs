use vulkano::device::Device;

use cgmath::{Matrix4, Vector3, Transform};

use crate::render::model::Model;
use crate::render::mymesh::MyMesh;

use std::path::Path;
use std::sync::Arc;

pub struct Lap {
  pub model: Model,
}

impl Lap {
  pub fn new(device: &Arc<Device>) -> Self {
    let mut mesh = MyMesh::from_gltf(Path::new("models/lep.glb"), true);
    mesh.update_transform_2(Vector3::<f32>::new(0.0, 0.0, 0.0), Matrix4::one(), [1.0/170.0, 1.0/170.0, 1.0/170.0]);
    let model = mesh.get_buffers(device);
    Lap {
      model
    }
  }
}
