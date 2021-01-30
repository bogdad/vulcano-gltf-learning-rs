use vulkano::device::Device;
use cgmath::Point3;
use std::sync::Arc;

use crate::actor::Actor;
use crate::render::{Model};
use crate::things::terrain_generation;

pub struct Sky {
  xs: f32,
  xe: f32,
  zs: f32,
  ze: f32,
  grid: [[Option<Model>; 3]; 3],
}

impl Sky {
  const L: f32 = 2.0;
  const W: f32 = 2.0;

  pub fn new(device: &Arc<Device>, x: f32, z: f32) -> Self {
    let current = terrain_generation::execute(128, Sky::L as i32).get_buffers(device);
    let mut grid: [[Option<Model>; 3]; 3] = Default::default();
    grid[1][1] = Some(current);
    Sky {
      grid,
      xs: x,
      xe: x + Sky::W,
      zs: z,
      ze: z + Sky::L,
    }
  }

  pub fn get_current(&self) -> &Model {
    &self.grid[1][1].as_ref().unwrap()
  }

  pub fn camera_entered(&mut self, pos: &Point3<f32>) {
    let L2 = Sky::L / 2.0;
    let W2 = Sky::L / 2.0;
    // 0,o -> (xs + xe)/2
    //
    /*
     * (pos.x - xc)/L , (pos.z - zc)/W ) + (1,1)
     */
  }



  fn get_ind(self, pos: &Point3<f32>) -> (usize, usize) {
    let xc = (self.xs + self.xe)/2.0;
    let zc = (self.zs + self.ze)/2.0;
    (((pos.x - xc)/Sky::L as isize) + 1, ((pos.z - zc)/Sky::W)  as isize + 1)
  }
}

fn not_more(x: f32, t: f32) {
  abs(x - t)
}

impl Actor for Sky {
  fn get_model(&self, device: &Arc<Device>) ->  &Model {
    self.get_current()
  }
}

mod tests {



}
