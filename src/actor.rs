use std::sync::Arc;
use vulkano::device::Device;

use crate::render::model::ModelScene;

pub trait Actor {
  fn get_model(&self, device: &Arc<Device>) -> Vec<ModelScene>;
}
