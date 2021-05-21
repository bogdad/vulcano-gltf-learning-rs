use vulkano::device::Device;

use cgmath::{Matrix4, Transform, Vector3};

use crate::render::model::Model;
use crate::render::mymesh::from_gltf;
use crate::render::mymesh::MyMesh;

use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct LapMesh {
  pub mesh: MyMesh,
}

pub struct Lap {
  mesh: LapMesh,
  pub model: Model,
}

impl LapMesh {
  pub fn new() -> Self {
    println!("lap: before");
    let mut mesh = from_gltf(Path::new("models/lep.glb"), true);
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
    LapMesh { mesh }
  }
}

impl Lap {
  pub fn new(device: &Arc<Device>) -> Self {
    let lap_mesh = LapMesh::new();
    let model = lap_mesh.mesh.get_buffers(device);
    Lap {
      mesh: lap_mesh,
      model,
    }
  }
}
