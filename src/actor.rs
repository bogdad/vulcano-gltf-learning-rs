use vulkano::device::Device;
use std::sync::Arc;

use crate::render::Model;
use crate::world::World;

pub trait Actor {
    fn get_model(&self, device: &Arc<Device>) -> Vec<Model>;
}
