use vulkano::device::Device;
use cgmath::{Point3, Vector2, VectorSpace, EuclideanSpace, Zero};

use futures::future::{RemoteHandle, FutureExt, Shared, BoxFuture, ready};
use futures::{Future};
use std::sync::Arc;
use std::mem;
use std::cmp::Ordering;
use std::sync::Mutex;

use crate::actor::Actor;
use crate::render::{Model};
use crate::things::terrain_generation;
use crate::executor::Executor;

pub struct Sky {
  inner: Arc<Mutex<SkyInner>>,
}

impl Sky {
  pub fn new(device: &Arc<Device>, x: f32, z: f32) -> Self {
    Sky {
      inner: Arc::new(Mutex::new(SkyInner::new(device, x, z))),
    }
  }

  pub fn get_current(&self) -> Vec<Model> {
    let locked = self.inner.lock();
    locked.unwrap().get_current()
  }

  pub fn tick(&mut self, executor: &Executor) {
    let locked = self.inner.lock();
    locked.unwrap().tick(self, executor);
  }

  pub fn camera_entered(&mut self, pos: &Point3<f32>) {
    let locked = self.inner.lock();
    locked.unwrap().camera_entered(pos);
  }
}

struct SkyInner {
  device: Arc<Device>,
  grid: Vec<Option<Model>>,
  waiting: Vec<Option<Shared<RemoteHandle<Option<Model>>>>>,
  x: Vector2<f32>,
  z: Vector2<f32>,
  // last seen camera position
  c: Vector2<f32>,
  prev_was_half: bool,
  ordered_cells: Vec<(usize, usize)>,
}

fn xindex(base:f32, step: usize) -> f32 {
  SkyInner::X * (step as f32) + base
}

fn zindex(base:f32, step: usize) -> f32 {
  SkyInner::Z * (step as f32) + base
}

fn ii(xi: usize, zi: usize) -> usize {
   zi * SkyInner::X_ROWS + xi
}

fn tii(t: &(usize, usize)) -> usize {
  ii(t.0, t.1)
}

fn gii(xi: isize, zi: isize) -> Option<usize> {
  let (cx, cz) = (crop(xi + SkyInner::MX as isize, SkyInner::X_ROWS),
    crop(zi + SkyInner::MZ as isize, SkyInner::Z_ROWS));
  if cx == None || cz == None {
    return None;
  }
  Some(ii(cx.unwrap(), cz.unwrap()))
}

fn giiu(xi: isize, zi: isize) -> usize {
  gii(xi, zi).unwrap()
}

fn crop(x: isize, bound: usize) -> Option<usize> {
  if x >= (bound as isize) || x < 0 {
    return None;
  }
  Some(x as usize)
}

impl SkyInner {
  const X: f32 = 4.0;
  const Z: f32 = 4.0;
  const X_ROWS: usize = 9;
  const Z_ROWS: usize = 9;
  const MX: usize = 4;
  const MZ: usize = 4;


  pub fn new(device: &Arc<Device>, x: f32, z: f32) -> Self {
    let mut grid: Vec<Option<Model>> = vec![];
    let mut waiting: Vec<Option<Shared<RemoteHandle<Option<Model>>>>> = vec![];
    for _i in 0..(SkyInner::X_ROWS * SkyInner::Z_ROWS) {
      grid.push(None);
      waiting.push(None);
    }
    let mut ordered: Vec<(usize, usize)> = vec![];
    for zi in 0..SkyInner::X_ROWS {
      for xi in 0..SkyInner::Z_ROWS {
        let try_cell = (xi, zi);
        ordered.push(try_cell);
      }
    }
    ordered.sort_by(|a, b| {
      let s_a = a.0 + a.1;
      let s_b = b.0 + b.1;
      if s_a < s_b {
        return Ordering::Less;
      }
      if s_a > s_b {
        return Ordering::Greater;
      }
      let c0 = a.0.cmp(&b.0);
      if c0 != Ordering::Equal {
        return c0;
      }
      return a.1.cmp(&b.1);
    });
    println!("odered ordered {:?}", ordered);
    SkyInner {
      device: Arc::clone(device),
      grid,
      waiting,
      x: Vector2::new(x, x + SkyInner::X),
      z: Vector2::new(z, z + SkyInner::Z),
      c: Vector2::new(0.0, 0.0),
      prev_was_half: false,
      ordered_cells: ordered,
    }
  }

  pub fn tick(&mut self, sky: &Sky, executor: &Executor) {
    let ahead_div = 3.0;
    let x_ahead = SkyInner::X / ahead_div;
    let z_ahead = SkyInner::Z / ahead_div;

    let peaked = &self.grid[giiu(0, 0)];
    if peaked.is_none() {
      let mut prev = mem::replace(&mut self.waiting[giiu(0, 0)], None);
      if prev.is_none() {
        self.waiting[giiu(0, 0)] = Some(
          executor.do_background(self.spawn_region(sky, self.x.x, self.z.x, false))
          .shared());
        prev = mem::replace(&mut self.waiting[giiu(0, 0)], None);
      }
      println!("blocking on sky");
      self.grid[giiu(0, 0)] = executor.wait(prev.unwrap());
      self.waiting[giiu(0, 0)] = None;
    }
    let indices = self.real_inds(SkyInner::X, SkyInner::Z);
    let half_indices = self.real_inds(x_ahead, z_ahead);
    if half_indices != (0, 0) {
      // spawn ahead of time model creation
      for try_cell in &self.ordered_cells {
        if self.waiting[tii(try_cell)].is_none() {
          let xx = xindex(self.x.x, try_cell.0);
          let zz = zindex(self.z.x, try_cell.1);
          self.waiting[tii(try_cell)] = Some(
            executor.do_background(
              self.spawn_region(sky, xx, zz, true)).shared()
            );
        }
      }
    } else {
        self.prev_was_half = false;
    }
    if indices != (0, 0) {
        println!("indices {:?} x {:?} z {:?} c {:?}",
          indices, self.x, self.z, self.c);
        // change the current
        // move each item in the grid in the right direction
        // negative index means we are moving existing items positively
        // when we are moving existing items positively we start from furthest
        println!("grid {:?}", self.grid.iter().map(|e| e.is_some()).collect::<Vec<_>>());
        println!("waiting {:?}", self.waiting.iter().map(|e| e.is_some()).collect::<Vec<_>>());
        let zrange:Vec<usize> = if indices.1 < 0 {
          (0..SkyInner::Z_ROWS).rev().collect()}
          else {
            (0..SkyInner::Z_ROWS).collect()};
        for zt in zrange {
          let xrange:Vec<usize> = if indices.0 < 0 {
            (0..SkyInner::X_ROWS).rev().collect()}
            else {
              (0..SkyInner::X_ROWS).collect()};
          for xt in xrange {
            let (zs, xs) = (crop(zt as isize + indices.1, SkyInner::Z_ROWS),
              crop(xt as isize + indices.0, SkyInner::X_ROWS));
            println!("moving {:?} {:?}", (xt, zt), (xs, zs));
            if zs == None || xs == None {
              self.grid[ii(xt, zt)] = None;
              self.waiting[ii(xt, zt)] = None;
            } else {
              self.grid.swap(ii(xt, zt), ii(xs.unwrap(), zs.unwrap()));
              self.waiting.swap(ii(xt, zt), ii(xs.unwrap(), zs.unwrap()));
            }
          }
        }
        println!("grid {:?}", self.grid.iter().map(|e| e.is_some()).collect::<Vec<_>>());
        println!("waiting {:?}", self.waiting.iter().map(|e| e.is_some()).collect::<Vec<_>>());

        self.x += Vector2::new(SkyInner::X * indices.0 as f32, SkyInner::X * indices.0 as f32);
        self.z += Vector2::new(SkyInner::Z * indices.1 as f32, SkyInner::Z * indices.1 as f32);
        println!("changing x {:?} z {:?}", self.x, self.z);
    }
  }


  pub fn get_current(&self) -> Vec<Model> {
    let mut res: Vec<Model> = vec![self.grid[giiu(0, 0)].as_ref().unwrap().clone()];
    if let Some(elem) = &self.grid[giiu(1, 0)] {
      res.push(elem.clone());
    };
    if let Some(elem) = &self.grid[giiu(-1, 0)] {
      res.push(elem.clone());
    };
    if let Some(elem) = &self.grid[giiu(0, 1)] {
      res.push(elem.clone());
    };
    if let Some(elem) = &self.grid[giiu(0, -1)] {
      res.push(elem.clone());
    };
    if let Some(elem) = &self.grid[giiu(-1, -1)] {
      res.push(elem.clone());
    };
    if let Some(elem) = &self.grid[giiu(-1, 1)] {
      res.push(elem.clone());
    };
    if let Some(elem) = &self.grid[giiu(1, -1)] {
      res.push(elem.clone());
    };
    if let Some(elem) = &self.grid[giiu(1, 1)] {
      res.push(elem.clone());
    };
    res
  }

  pub fn camera_entered(&mut self, pos: &Point3<f32>) {
    self.c = Vector2::new(pos.x, pos.z);
  }


  // indices in the grid. assumption is square [(xs, xe),(zs, ze)] is the central square in the grid
  fn real_inds(&self, l: f32, w: f32) -> (isize, isize) {
    let gc = Vector2::new((self.x.x + self.x.y)/2.0, (self.z.x + self.z.y)/2.0);
    //println!("x {:?} z {:?} gc {:?}", self.x, self.z, gc);
    (((self.c.x - gc.x)/l) as isize, ((self.c.y - gc.y)/l) as isize)
  }

  fn spawn_region(&self, sky: &Sky, x: f32, z: f32, call_back: bool) -> Shared<impl Future<Output=Option<Model>>> {
    let weak_device = Arc::downgrade(&self.device);
    let weak_self = Arc::downgrade(&sky.inner);
    let fut = async move {
      println!("generated ({:?},{:?})", x, z);
      if let Some(device) = weak_device.upgrade() {
        println!("device upgraded");
        if let Some(selv) = weak_self.upgrade() {
          println!("selv upgraded");
          let res = terrain_generation::execute(32, SkyInner::X as i32, x, z).get_buffers(&device);
          if call_back {
            accept_results(selv);
          }
          Some(res)
        } else {
          None
        }
      } else {
        None
      }
    };
    fut.shared()
  }
}

  fn accept_results(sky: Arc<Mutex<SkyInner>>) {
    let mut selv = sky.lock().unwrap();
    for zi in 0..SkyInner::X_ROWS {
      for xi in 0..SkyInner::Z_ROWS {
        println!("trying waiting");
        if let Some(shared) = &selv.waiting[ii(xi, zi)] {
          if let Some(model_opt) = shared.peek() {
            println!("results accepted");
            selv.grid[ii(xi, zi)] = model_opt.clone();
          }
        }
      }
    }
  }

impl Actor for Sky {
  fn get_model(&self, device: &Arc<Device>) ->  Vec<Model> {
    self.get_current()
  }
}

mod tests {



}
