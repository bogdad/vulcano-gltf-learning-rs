use crate::world::Mode;
use cgmath::{Angle, EuclideanSpace, InnerSpace, Matrix3, Matrix4, Point3, Rad, Vector3};

use winit::dpi::PhysicalPosition;
use winit::event::{KeyboardInput, VirtualKeyCode};

use std::fmt;

use crate::shaders;
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
  pub last_x: Option<f64>,
  pub last_y: Option<f64>,
  pub yaw: f32,
  pub pitch: f32,
}

impl Camera {
  fn adjust(&mut self, mode: Mode, by: Vector3<f32>) {
    match mode {
      Mode::MoveCameraPos => self.pos += by,
      Mode::MoveCameraFront => self.front += by,
      Mode::MoveCameraUp => self.up += by,
    }
  }

  pub fn react_mouse(&mut self, position: &PhysicalPosition<f64>) {
    if let Some(lx) = self.last_x {
      if let Some(ly) = self.last_y {
        let mut xoffset: f32 = (position.x - lx) as f32;
        let mut yoffset: f32 = (ly - position.y) as f32; // reversed since y-coordinates range from bottom to top
        let sensitivity = 0.1;
        xoffset *= sensitivity;
        yoffset *= sensitivity;
        self.yaw += xoffset;
        self.pitch += yoffset;
        if self.pitch > 89.0 {
          self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
          self.pitch = -89.0;
        }
        let direction = Vector3::new(
          Rad(self.yaw).cos() * Rad(self.pitch).cos(),
          Rad(self.pitch).sin(),
          Rad(self.yaw).sin() * Rad(self.pitch).cos(),
        );
        self.front = direction.normalize();
      }
    }
    self.last_x = Some(position.x);
    self.last_y = Some(position.y);
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

  pub fn proj(&self, graph: &Graph) -> shaders::main::vs::ty::Data {
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
    shaders::main::vs::ty::Data {
      //world: Matrix4::from(eye).into(),
      world: Matrix4::from(rotation).into(),
      //world: <Matrix4<f32> as One>::one().into(),
      view: (view * scale).into(),
      proj: proj.into(),
      camera_position: self.pos.into(),
    }
  }

  pub fn proj_skybox(&self, graph: &Graph) -> shaders::skybox::vs::ty::Data {
    //let _elapsed = self.rotation_start.elapsed();
    let rotation = 0;
    //elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
    let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

    // note: this teapot was meant for OpenGL where the origin is at the lower left
    //       instead the origin is at the upper left in, Vulkan, so we reverse the Y axis
    let aspect_ratio = graph.dimensions[0] as f32 / graph.dimensions[1] as f32;
    let mut proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.001, 100000.0);

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
    shaders::skybox::vs::ty::Data {
      //world: Matrix4::from(eye).into(),
      world: Matrix4::from(rotation).into(),
      //world: <Matrix4<f32> as One>::one().into(),
      view: (view * scale).into(),
      proj: proj.into(),
      camera_position: self.pos.into(),
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
