use std::sync::Arc;
use vulkano::device::Device;

use crate::render::Model;

pub trait Actor {
  fn get_model(&self, device: &Arc<Device>) -> Vec<Model>;
}
