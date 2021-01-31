use vulkano::device::Device;
use cgmath::Point3;
use futures::future::{FutureExt, Shared, BoxFuture, ready};
use futures::Future;
use std::sync::Arc;
use std::pin::Pin;

use crate::actor::Actor;
use crate::render::{Model};
use crate::things::terrain_generation;

pub struct Sky<'a> {

  device: Arc<Device>,
  xs: f32,
  xe: f32,
  zs: f32,
  ze: f32,
  grid: Vec<Shared<BoxFuture<'a, Option<Model>>>>,
}

  fn ii(xi: usize, zi: usize) -> usize {
    xi*3 + zi
  }

impl Sky<'_> {
  const L: f32 = 2.0;
  const W: f32 = 2.0;



  pub fn new(device: &Arc<Device>, x: f32, z: f32) -> Self {
    let current = terrain_generation::execute(128, Sky::L as i32).get_buffers(device);
    let mut grid: Vec<Shared<BoxFuture<Option<Model>>>> = vec![];
    for i in 0..9 {
      grid.push(ready(None).boxed().shared());
    }
    grid[ii(1,1)] = ready(Some(current)).boxed().shared();
    Sky {
      device: Arc::clone(device),
      grid,
      xs: x,
      xe: x + Sky::W,
      zs: z,
      ze: z + Sky::L,
    }
  }



  pub fn get_current(&self) -> &Model {
    &self.grid[ii(1, 1)].peek().unwrap().as_ref().unwrap()
  }

  pub fn camera_entered(&mut self, pos: &Point3<f32>) {
    let l2 = Sky::L / 2.0;
    let w2 = Sky::W / 2.0;
    let indices = self.real_inds(pos, Sky::L, Sky::W);
    if indices != (0, 0) {
        // change the current

    }
    let half_indices = self.real_inds(pos, l2, w2);
    if half_indices != (0, 0) {
        // spawn ahead of time model creation
    }
  }


  // indices in the grid. assumption is square [(xs, xe),(zs, ze)] is the central square in the grid
  fn real_inds(&self, pos: &Point3<f32>, l: f32, w: f32) -> (isize, isize) {
    let xc = (self.xs + self.xe)/2.0;
    let zc = (self.zs + self.ze)/2.0;
    ((((pos.x - xc)/l) as isize), (((pos.z - zc)/w)  as isize))
  }

  fn spawn_region(&self, xi: isize, zi: isize) -> Shared<impl Future<Output=Option<Model>>> {
    let weak_device = Arc::downgrade(&self.device);
    let fut = async move {
      if let Some(device) = weak_device.upgrade() {
        Some(terrain_generation::execute(128, Sky::L as i32).get_buffers(&device))
      } else {
        None
      }
    };
    fut.shared()
  }
}

impl Actor for Sky<'_> {
  fn get_model(&self, device: &Arc<Device>) ->  &Model {
    self.get_current()
  }
}

mod tests {



}
