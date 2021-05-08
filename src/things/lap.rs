use vulkano::device::Device;

use cgmath::{Matrix4, Transform, Vector3};

use crate::render::model::Model;
use crate::render::mymesh::MyMesh;

use std::path::Path;
use std::sync::Arc;

pub struct Lap {
  pub model: Model,
}

impl Lap {
  pub fn new(device: &Arc<Device>) -> Self {
    println!("lap: before");
    let mut mesh = MyMesh::from_gltf(Path::new("models/lep.glb"), true);
    println!("lap: before update");
    mesh.update_transform_2(
      Vector3::<f32>::new(-15.0, -3.0, 0.0),
      Matrix4::one(),
      [1.0, 1.0, 1.0],
    );
    mesh.map_vertex(|v| {
      v.x = -v.x / 500.0;
      v.y = v.y / 500.0;
      v.z = v.z / 500.0;
      std::mem::swap(&mut v.y, &mut v.x);
    });
    println!("lap: after");
    let model = mesh.get_buffers(device);
    Lap { model }
  }
}
