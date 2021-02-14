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
    let texture_size = texts.size();
    let mesh = PrimitiveTriangle::new_tex(pos,
      Point2::new(info.min.0 as f32, info.min.1 as f32),
      Point2::new(info.max.0 as f32, info.max.1 as f32),
      texture_size,
    );
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
