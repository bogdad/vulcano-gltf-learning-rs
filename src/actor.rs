use vulkano::device::Device;
use std::sync::Arc;

use crate::render::model::Model;

pub trait Actor {
    fn get_model(&self, device: &Arc<Device>) -> Vec<Model>;
}
