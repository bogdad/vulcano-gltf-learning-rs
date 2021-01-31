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
    let W2 = Sky::W / 2.0;
    let indices = self.real_inds(pos, Sky::L, Sky::W);
    if (indices != (0, 0)) {

    }
    let half_indices = self.real_inds(pos, L2, W2);
    if half_indices != (0, 0) {

    }
  }


  // indices in the grid. assumption is square [(xs, xe),(zs, ze)] is the central square in the grid
  fn real_inds(self, pos: &Point3<f32>, l: f32, w: f32) -> (isize, isize) {
    let xc = (self.xs + self.xe)/2.0;
    let zc = (self.zs + self.ze)/2.0;
    ((((pos.x - xc)/l) as isize), (((pos.z - zc)/w)  as isize))
  }
}

impl Actor for Sky {
  fn get_model(&self, device: &Arc<Device>) ->  &Model {
    self.get_current()
  }
}

mod tests {



}
