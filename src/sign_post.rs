use vulkano::device::Device;
use cgmath::Point3;

use std::sync::Arc;

use crate::things::primitives::PrimitiveTriangle;
use crate::render::model::Model;

pub struct SignPost {
  device: Arc<Device>,
  mesh: PrimitiveTriangle,
  model: Model,
  text: String,
}

impl SignPost {
  pub fn new(device: &Arc<Device>, pos: Point3<f32>, text: String) -> Self {
    let mesh = PrimitiveTriangle::new(pos);
    let model = mesh.mesh.get_buffers(device);
    SignPost {
      device: device.clone(),
      mesh,
      model,
      text,
    }
  }

  pub fn get_model(&self) -> &Model {
    &self.model
  }
}
