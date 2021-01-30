
use winit::event::{VirtualKeyCode, KeyboardInput};
use cgmath::Point3;
use std::fmt;

use crate::sky::Sky;
use crate::render::Model;
use crate::Graph;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mode {
  MoveCameraPos,
  MoveCameraFront,
  MoveCameraUp,
}

impl Mode {
  const VALUES: [Self; 3] = [Self::MoveCameraPos, Self::MoveCameraFront, Self::MoveCameraUp];
  fn next(&self) -> Mode {
    let mut prev = Self::MoveCameraUp;
    for mode in Mode::VALUES.iter().copied() {
        if prev == *self {
          return mode;
        }
        prev = mode;
    }
    return prev
  }
}

pub struct World {
  pub mode: Mode,
  sky: Sky,
}
impl World {
  pub fn new(graph: &Graph) -> Self {
    let sky = Sky::new(&graph.device, 2.0, 2.0);
    World {
      mode: Mode::MoveCameraPos,
      sky,
    }
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
      virtual_keycode: Some(key_code),
      ..
    } = input {
      match key_code {
        VirtualKeyCode::Escape => self.command(),
        _ => (),
      }
    }
  }

  pub fn get_models(&self) -> Vec<&Model> {
    [self.sky.get_current()].to_vec()
  }
}

impl fmt::Display for World {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "mode {:?}", self.mode)
  }
}
