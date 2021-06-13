use vulkano::device::Device;

use cgmath::{Matrix4, Rad, Vector3};

use crate::render::from_gltf;
use crate::render::Model;
use crate::render::MyMesh;

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
    let mut mesh = from_gltf(Path::new("models/lep.glb"), false);
    mesh.reset_transform();
    mesh.update_transform_2(
      Vector3::<f32>::new(0.0, 0.0, 0.0),
      //Matrix4::one(),
      Matrix4::from_angle_z(-Rad(std::f32::consts::FRAC_PI_2)),
      [1.0 / 500.0, 1.0 / 500.0, 1.0 / 500.0],
    );
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
