use vulkano::device::Device;
use cgmath::{Point3, Point2};

use std::sync::Arc;

use crate::things::texts::Texts;
use crate::things::primitives::PrimitiveTriangle;
use crate::render::model::Model;

pub struct SignPost {
  device: Arc<Device>,
  mesh: PrimitiveTriangle,
  model: Model,
  text: String,
}

impl SignPost {
  pub fn new(device: &Arc<Device>, pos: Point3<f32>, text: String, texts: &Texts) -> Self {
    let info = texts.info(&text);
    let mesh = PrimitiveTriangle::newTex(pos, Point2::new(info.rect.0 as f32, info.rect.1 as f32));
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
