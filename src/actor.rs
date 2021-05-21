use std::sync::Arc;
use vulkano::device::Device;

use crate::render::model::Model;
use crate::render::scene::Scene;

pub trait Actor {
  fn get_model(&self, device: &Arc<Device>) -> Vec<Model>;
}
