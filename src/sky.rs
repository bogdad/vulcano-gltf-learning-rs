use cgmath::{Point3, Vector2};
use parking_lot::MappedRwLockReadGuard;
use parking_lot::RwLockReadGuard;
use vulkano::device::Device;

use futures::executor::block_on;
use futures::future::RemoteHandle;
use parking_lot::RwLock;
use std::cmp::Ordering;
use std::sync::Arc;

use crate::actor::Actor;
use crate::executor::Executor;
use crate::render::model::Model;
use crate::render::scene::Scene;
use crate::shaders::main::fs;
use crate::things::terrain_generation;
use crate::things::terrain_generation::TerrainModel;

fn xindex(base: f32, step: isize) -> f32 {
  Sky::X * (step as f32) + base
}

fn zindex(base: f32, step: isize) -> f32 {
  Sky::Z * (step as f32) + base
}

fn ii(xi: usize, zi: usize) -> usize {
  zi * Sky::X_ROWS + xi
}

fn tii(t: &(isize, isize)) -> usize {
  gii(t.0, t.1).unwrap()
}

fn gii(xi: isize, zi: isize) -> Option<usize> {
  let (cx, cz) = (
    crop(xi + Sky::MX as isize, Sky::X_ROWS),
    crop(zi + Sky::MZ as isize, Sky::Z_ROWS),
  );
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

type ArcCacheCellInner = Arc<RwLock<CacheCellInner>>;

#[derive(Default)]
struct CacheCellInner {
  model: Option<TerrainModel>,
}

#[derive(Default)]
struct CacheCell {
  inner: ArcCacheCellInner,
  future: Option<RemoteHandle<()>>,
}

fn get_border_vec(
  cell_opt: Option<ArcCacheCellInner>,
  f: fn(&TerrainModel) -> Vec<f32>,
) -> Option<Vec<f32>> {
  if cell_opt.is_none() {
    return None;
  }
  let cell = cell_opt.unwrap();
  let read = cell.read();
  let tm: Option<&TerrainModel> = ((*read).model).as_ref();
  Some(f(&tm.unwrap()))
}

impl CacheCell {
  fn is_queued(&self) -> bool {
    self.future.is_some()
  }

  fn spawn_region(
    &mut self,
    executor: &Executor,
    device: &Arc<Device>,
    x: f32,
    z: f32,
    oleft: Option<ArcCacheCellInner>,
    oright: Option<ArcCacheCellInner>,
    otop: Option<ArcCacheCellInner>,
    obottom: Option<ArcCacheCellInner>,
  ) {
    if self.future.is_some() {
      return;
    }
    {
      let read_locked = self.inner.read();
      if read_locked.model.is_some() {
        return;
      }
    }

    let weak_device = Arc::downgrade(device);
    let weak_self_inner = Arc::downgrade(&self.inner);
    let fut = async move {
      // println!("generated ({:?},{:?})", x, z);
      if let Some(device) = weak_device.upgrade() {
        if let Some(self_inner) = weak_self_inner.upgrade() {
          let mut locked = self_inner.write();
          if locked.model.is_some() {
            return;
          }
          let vleft = get_border_vec(oleft, |tm| tm.right.clone());
          let vright = get_border_vec(oright, |tm| tm.left.clone());
          let vtop = get_border_vec(otop, |tm| tm.bottom.clone());
          let vbottom = get_border_vec(obottom, |tm| tm.top.clone());
          let res = terrain_generation::execute(
            &device,
            8,
            Sky::X as i32,
            x + Sky::X / 2.0,
            z + Sky::Z / 2.0,
            vleft,
            vright,
            vtop,
            vbottom,
          );
          locked.model = Some(res);
        }
      }
    };
    let pinned = Box::pin(fut);
    self.future = Some(executor.do_background(pinned));
  }

  fn block(&mut self) {
    {
      let read_locked = self.inner.read();
      if read_locked.model.is_some() {
        return;
      }
    }
    let future = self.future.take();
    block_on(future.unwrap())
  }

  fn create_block(
    &mut self,
    executor: &Executor,
    device: &Arc<Device>,
    x: f32,
    z: f32,
    oleft: Option<ArcCacheCellInner>,
    oright: Option<ArcCacheCellInner>,
    otop: Option<ArcCacheCellInner>,
    obottom: Option<ArcCacheCellInner>,
  ) {
    {
      let peaked = self.inner.read();
      if peaked.model.is_some() {
        return;
      }
    }
    println!("blocking on sky");
    self.spawn_region(executor, device, x, z, oleft, oright, otop, obottom);
    self.block();
  }

  fn _status(&self) -> String {
    let model = {
      let read_locked = self.inner.read();
      read_locked.model.is_some()
    };
    format!(
      "cache cell model {:?} futures {:?}",
      model,
      self.future.is_some()
    )
  }

  fn model(&self) -> Option<TerrainModel> {
    let read_locked = self.inner.read();
    read_locked.model.clone()
  }
}

pub struct Sky {
  device: Arc<Device>,
  cache: Vec<CacheCell>,
  x: Vector2<f32>,
  z: Vector2<f32>,
  // last seen camera position
  c: Vector2<f32>,
  ordered_cells: Vec<(isize, isize)>,
}

impl Sky {
  const X: f32 = 4.0;
  const Z: f32 = 4.0;
  const X_ROWS: usize = 39;
  const Z_ROWS: usize = 39;
  const MX: usize = 19;
  const MZ: usize = 19;

  pub fn new(device: &Arc<Device>, x: f32, z: f32) -> Self {
    let mut cache: Vec<CacheCell> = vec![];
    for _i in 0..(Sky::X_ROWS * Sky::Z_ROWS) {
      cache.push(CacheCell::default());
    }
    let mut ordered: Vec<(isize, isize)> = vec![];
    for zi in 0..Sky::X_ROWS {
      for xi in 0..Sky::Z_ROWS {
        let try_cell: (isize, isize) = (
          xi as isize - Sky::MX as isize,
          zi as isize - Sky::MZ as isize,
        );
        ordered.push(try_cell);
      }
    }
    ordered.sort_by(|a, b| {
      let s_a = (a.0 + a.1).abs();
      let s_b = (b.0 + b.1).abs();
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
      a.1.cmp(&b.1)
    });
    //println!("odered ordered {:?}", ordered);
    Sky {
      device: Arc::clone(device),
      cache,
      x: Vector2::new(x, x + Sky::X),
      z: Vector2::new(z, z + Sky::Z),
      c: Vector2::new(0.0, 0.0),
      ordered_cells: ordered,
    }
  }

  fn get_arc(&self, cell: &(isize, isize)) -> Option<ArcCacheCellInner> {
    let ppp = gii(cell.0, cell.1);
    if ppp.is_none() {
      return None;
    }
    if !self.cache[ppp.unwrap()].is_queued() {
      None
    } else {
      Some(self.cache[tii(cell)].inner.clone())
    }
  }

  pub fn tick(&mut self, executor: &Executor) {
    let ahead_div = 3.0;
    let x_ahead = Sky::X / ahead_div;
    let z_ahead = Sky::Z / ahead_div;

    self.cache[giiu(0, 0)].create_block(
      executor,
      &self.device,
      self.x.x,
      self.z.x,
      None,
      None,
      None,
      None,
    );

    let indices = self.real_inds(Sky::X, Sky::Z);
    let half_indices = self.real_inds(x_ahead, z_ahead);
    if half_indices != (0, 0) {
      // spawn ahead of time model creation
      let ordered_cells = &self.ordered_cells;
      for try_cell in ordered_cells {
        if !self.cache[tii(try_cell)].is_queued() {
          let xx = xindex(self.x.x, try_cell.0);
          let zz = zindex(self.z.x, try_cell.1);
          let try_left = (try_cell.0 - 1, try_cell.1);
          let try_right = (try_cell.0 + 1, try_cell.1);
          let try_top = (try_cell.0, try_cell.1 - 1);
          let try_bottom = (try_cell.0, try_cell.1 + 1);
          let oleft = self.get_arc(&try_left);
          let oright = self.get_arc(&try_right);
          let otop = self.get_arc(&try_top);
          let obottom = self.get_arc(&try_bottom);
          self.cache[tii(try_cell)].spawn_region(
            executor,
            &self.device,
            xx,
            zz,
            oleft,
            oright,
            otop,
            obottom,
          );
        }
      }
    }
    if indices != (0, 0) {
      //println!("indices {:?} x {:?} z {:?} c {:?}",
      //  indices, self.x, self.z, self.c);
      // change the current
      // move each item in the grid in the right direction
      // negative index means we are moving existing items positively
      // when we are moving existing items positively we start from furthest
      let zrange: Vec<usize> = if indices.1 < 0 {
        (0..Sky::Z_ROWS).rev().collect()
      } else {
        (0..Sky::Z_ROWS).collect()
      };
      for zt in zrange {
        let xrange: Vec<usize> = if indices.0 < 0 {
          (0..Sky::X_ROWS).rev().collect()
        } else {
          (0..Sky::X_ROWS).collect()
        };
        for xt in xrange {
          let (xs, zs) = (
            crop(xt as isize + indices.0, Sky::X_ROWS),
            crop(zt as isize + indices.1, Sky::Z_ROWS),
          );
          //println!("moving {:?} {:?}", (xt, zt), (xs, zs));
          if zs == None || xs == None {
            self.cache[ii(xt, zt)] = Default::default();
          } else {
            self.cache.swap(ii(xt, zt), ii(xs.unwrap(), zs.unwrap()));
          }
        }
      }
      /*println!(
        "cache {:?}",
        self.cache.iter().map(|e| e.status()).collect::<Vec<_>>()
      );*/

      self.x += Vector2::new(Sky::X * indices.0 as f32, Sky::X * indices.0 as f32);
      self.z += Vector2::new(Sky::Z * indices.1 as f32, Sky::Z * indices.1 as f32);
      // println!("changing x {:?} z {:?}", self.x, self.z);
    }
  }

  pub fn get_current(&self) -> Vec<(Model, Scene)> {
    let mut res: Vec<(Model, Scene)> = vec![self.cache[giiu(0, 0)]
      .model()
      .map(|tm| tm.model_scene())
      .as_ref()
      .unwrap()
      .clone()];
    for (i, j) in &self.ordered_cells {
      if i.abs() + j.abs() < 10 {
        if let Some(elem) = &self.cache[giiu(*i, *j)].model().map(|tm| tm.model_scene()) {
          res.push(elem.clone());
        };
      }
    }
    res[0].1.point_lights = vec![
      Arc::new(fs::ty::PointLight {
        position: [-100.0, -100.0, -1000.0],
        color: [1.0, 1.0, 1.0],
        intensity: 1000.0 * 1000.0,
        ..Default::default()
      }),
      Arc::new(fs::ty::PointLight {
        position: [-13.0, 10.0, -14.0],
        color: [1.0, 1.0, 0.0],
        intensity: 400.0,
        ..Default::default()
      }),
    ];
    res
  }

  pub fn camera_entered(&mut self, pos: &Point3<f32>) {
    self.c = Vector2::new(pos.x, pos.z);
  }

  // indices in the grid. assumption is square [(xs, xe),(zs, ze)] is the central square in the grid
  fn real_inds(&self, l: f32, _w: f32) -> (isize, isize) {
    let gc = Vector2::new((self.x.x + self.x.y) / 2.0, (self.z.x + self.z.y) / 2.0);
    //println!("x {:?} z {:?} gc {:?}", self.x, self.z, gc);
    (
      ((self.c.x - gc.x) / l) as isize,
      ((self.c.y - gc.y) / l) as isize,
    )
  }
}

impl Actor for Sky {
  fn get_model(&self, _device: &Arc<Device>) -> Vec<(Model, Scene)> {
    self.get_current()
  }
}

mod tests {}
