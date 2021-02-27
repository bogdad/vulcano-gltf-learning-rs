use cgmath::Point3;
use winit::event::{KeyboardInput, VirtualKeyCode};

use std::fmt;

use crate::executor::Executor;
use crate::render::model::ModelScene;
use crate::sign_post::SignPost;
use crate::sky::Sky;
use crate::Graph;
use crate::things::primitives::PrimitiveSkyBox;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mode {
  MoveCameraPos,
  MoveCameraFront,
  MoveCameraUp,
}

impl Mode {
  const VALUES: [Self; 3] = [
    Self::MoveCameraPos,
    Self::MoveCameraFront,
    Self::MoveCameraUp,
  ];
  fn next(&self) -> Mode {
    let mut prev = Self::MoveCameraUp;
    for mode in Mode::VALUES.iter().copied() {
      if prev == *self {
        return mode;
      }
      prev = mode;
    }
    prev
  }
}

pub struct World {
  executor: Executor,
  pub mode: Mode,
  sky: Sky,
  sky_box: PrimitiveSkyBox,
  sign_posts: Vec<SignPost>,
}
impl World {
  pub fn new(executor: Executor, graph: &Graph, sign_posts: Vec<SignPost>) -> Self {
    let sky = Sky::new(&graph.device, 0.0, 0.0);
    let sky_box = PrimitiveSkyBox::new(&graph.device);
    World {
      executor,
      mode: Mode::MoveCameraPos,
      sky,
      sky_box,
      sign_posts,
    }
  }

  pub fn tick(&mut self) {
    self.sky.tick(&self.executor);
  }

  pub fn camera_entered(&mut self, pos: &Point3<f32>) {
    // entering
    if pos.x.rem_euclid(2.0) < f32::EPSILON && pos.z.rem_euclid(2.0) < f32::EPSILON {
      //println!(" entering x, y, z {:?} {:?} {:?}", pos.x, pos.y, pos.z);
    }
    self.sky.camera_entered(pos);
  }

  pub fn command(&mut self) {
    self.mode = self.mode.next();
  }

  pub fn react(&mut self, input: &KeyboardInput) {
    if let KeyboardInput {
      virtual_keycode: Some(VirtualKeyCode::Escape),
      ..
    } = input
    {
      self.command();
    }
  }

  pub fn get_models(&self) -> Vec<ModelScene> {
    let mut res = vec![];
    res.extend(self.sky.get_current());
    res.extend(self.sky_box.get_model());
    for sign_post in &self.sign_posts {
      res.push((sign_post.get_model().clone(), Default::default()));
    }
    res
  }
}

impl fmt::Display for World {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "mode {:?}", self.mode)
  }
}
