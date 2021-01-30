use vulkano::device::Device;
use std::sync::Arc;

use crate::render::Model;

pub trait Actor {
    fn get_model(&self, device: &Arc<Device>) -> &Model;
}
