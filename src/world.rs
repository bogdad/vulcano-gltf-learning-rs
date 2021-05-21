use cgmath::Point3;
use winit::event::{KeyboardInput, VirtualKeyCode};

use std::fmt;

use crate::executor::Executor;
use crate::render::model::Model;
use crate::render::scene::Scene;
use crate::sign_post::SignPost;
use crate::sky::Sky;
use crate::things::primitives::PrimitiveSkyBox;
use crate::Graph;
use crate::Settings;

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
  settings: Settings,
  executor: Executor,
  pub mode: Mode,
  sky: Sky,
  sign_posts: Vec<SignPost>,
  skybox: PrimitiveSkyBox,
}
impl World {
  pub fn new(
    settings: Settings,
    executor: Executor,
    graph: &Graph,
    sign_posts: Vec<SignPost>,
  ) -> Self {
    let sky = Sky::new(settings.clone(), &graph.device, 0.0, 0.0);
    let skybox = PrimitiveSkyBox::new(&graph.device);
    World {
      settings,
      executor,
      mode: Mode::MoveCameraPos,
      sky,
      sign_posts,
      skybox,
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

  pub fn get_scenes(&self) -> Vec<&Scene> {
    let mut res = vec![];
    if self.settings.sky_enabled {
      res.extend(self.sky.get_scene());
    }
    res
  }

  pub fn get_models(&self) -> Vec<Model> {
    let mut res = vec![];
    if self.settings.sky_enabled {
      res.extend(self.sky.get_current());
    }
    if self.settings.letters_enabled {
      for sign_post in &self.sign_posts {
        res.push(sign_post.get_model());
      }
    }
    res
  }

  pub fn get_models_skybox(&self) -> Vec<&Model> {
    let mut res = vec![];
    res.extend(self.skybox.get_model());
    res
  }
}

impl fmt::Display for World {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "mode {:?}", self.mode)
  }
}
