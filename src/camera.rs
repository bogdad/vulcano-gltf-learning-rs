use crate::world::Mode;
use cgmath::{EuclideanSpace, InnerSpace, Matrix3, Matrix4, Point3, Rad, Vector3};

use winit::event::{KeyboardInput, VirtualKeyCode};

use std::fmt;

use crate::shaders::main::vs;
use crate::Graph;

#[derive(Debug)]
pub struct Camera {
  // where camera is looking at
  pub front: Vector3<f32>,
  // where camera is
  pub pos: Point3<f32>,
  // up is there
  pub up: Vector3<f32>,
  pub speed: f32,
}
impl Camera {
  fn adjust(&mut self, mode: Mode, by: Vector3<f32>) {
    match mode {
      Mode::MoveCameraPos => self.pos += by,
      Mode::MoveCameraFront => self.front += by,
      Mode::MoveCameraUp => self.up += by,
    }
  }

  pub fn react(self: &mut Camera, mode: Mode, input: &KeyboardInput) -> bool {
    if let KeyboardInput {
      virtual_keycode: Some(key_code),
      ..
    } = input
    {
      let camera_speed = self.speed;
      let zz = self.front.cross(self.up).normalize();
      match key_code {
        VirtualKeyCode::A => {
          self.adjust(mode, zz * camera_speed);
          return true;
        }
        VirtualKeyCode::D => {
          self.adjust(mode, -zz * camera_speed);
          return true;
        }
        VirtualKeyCode::W => {
          self.adjust(mode, camera_speed * self.front);
          return true;
        }
        VirtualKeyCode::S => {
          self.adjust(mode, -camera_speed * self.front);
          return true;
        }
        _ => {
          return false;
        }
      };
    }
    false
  }

  pub fn proj(&self, graph: &Graph) -> vs::ty::Data {
    //let _elapsed = self.rotation_start.elapsed();
    let rotation = 0;
    //elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
    let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

    // note: this teapot was meant for OpenGL where the origin is at the lower left
    //       instead the origin is at the upper left in, Vulkan, so we reverse the Y axis
    let aspect_ratio = graph.dimensions[0] as f32 / graph.dimensions[1] as f32;
    let mut proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.1, 100.0);

    // flipping the "horizontal" projection bit
    proj[0][0] = -proj[0][0];

    let target = self.pos.to_vec() + self.front;

    let view = Matrix4::look_at(self.pos, Point3::from_vec(target), self.up);
    let scale = Matrix4::from_scale(0.99);
    /*
       mat4 worldview = uniforms.view * uniforms.world;
       v_normal = transpose(inverse(mat3(worldview))) * normal;
       gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
    */
    vs::ty::Data {
      //world: Matrix4::from(eye).into(),
      world: Matrix4::from(rotation).into(),
      //world: <Matrix4<f32> as One>::one().into(),
      view: (view * scale).into(),
      proj: proj.into(),
    }
  }
}

impl fmt::Display for Camera {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "pos: ({}, {}, {}) front: ({}, {}, {}), up: ({}, {}, {}) speed: {}",
      self.pos.x,
      self.pos.y,
      self.pos.z,
      self.front.x,
      self.front.y,
      self.front.z,
      self.up.x,
      self.up.y,
      self.up.z,
      self.speed
    )
  }
}
