use cgmath::{Point2, Point3};
use vulkano::device::Device;

use std::sync::Arc;

use crate::render::model::Model;
use crate::things::primitives::PrimitiveTriangle;
use crate::things::texts::Texts;

pub struct SignPost {
  model: Model,
}

impl SignPost {
  pub fn new(device: &Arc<Device>, pos: Point3<f32>, text: String, texts: &Texts) -> Self {
    let info = texts.info(&text);
    let texture_size = texts.size();
    let mesh = PrimitiveTriangle::new_tex(
      pos,
      Point2::new(info.min.0 as f32, info.min.1 as f32),
      Point2::new(info.max.0 as f32, info.max.1 as f32),
      texture_size,
    );
    let model = mesh.mesh.get_buffers(device);
    SignPost { model }
  }

  pub fn get_model(&self) -> Model {
    self.model.clone()
  }
}
