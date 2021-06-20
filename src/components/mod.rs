use bevy_ecs::bundle::Bundle;
use cgmath::{Array, Point2, Point3, Vector3};

#[derive()]
pub struct Position {
  pub point3: Point3<f32>,
}

impl Default for Position {
  fn default() -> Position {
    Position {
      point3: Point3::from_value(0.0),
    }
  }
}

pub struct Velocity {
  pub vec3: Vector3<f32>,
}

impl Default for Velocity {
  fn default() -> Velocity {
    Velocity {
      vec3: Vector3::from_value(0.0),
    }
  }
}

#[derive()]
pub struct CameraId {
  pub front: Vector3<f32>,
  pub up: Vector3<f32>,
  pub pitch: f32,
  pub yaw: f32,
  pub speed: f32,
  pub last_x: Option<f32>,
  pub last_y: Option<f32>,
  pub last_wheel_x: Option<f32>,
  pub last_wheel_y: Option<f32>,
}

impl Default for CameraId {
  fn default() -> CameraId {
    CameraId {
      front: Vector3::from_value(0.0),
      up: Vector3::from_value(0.0),
      pitch: 0.0,
      yaw: 0.0,
      speed: 0.0,
      last_x: None,
      last_y: None,
      last_wheel_x: None,
      last_wheel_y: None,
    }
  }
}

pub struct Acceleration {
  pub vec3: Vector3<f32>,
}

impl Default for Acceleration {
  fn default() -> Acceleration {
    Acceleration {
      vec3: Vector3::from_value(0.0),
    }
  }
}

impl Acceleration {}

#[derive(Bundle, Default)]
pub struct CameraBundle {
  pub camera: CameraId,
  pub position: Position,
  pub velocity: Velocity,
  pub accel: Acceleration,
}

#[derive(Default, Debug)]
pub struct KeyboardState {
  pub a: bool,
  pub d: bool,
  pub w: bool,
  pub s: bool,
  pub q: bool,
  pub e: bool,
  pub z: bool,
  pub x: bool,
  pub c: bool,
  pub cmd: bool,
}

#[derive(Debug)]
pub struct MouseState {
  pub position: Point2<f32>,
  pub scroll_position: Point2<f32>,
}

impl Default for MouseState {
  fn default() -> MouseState {
    MouseState {
      position: Point2::from_value(0.0),
      scroll_position: Point2::from_value(0.0),
    }
  }
}

#[derive(Default, Debug)]
pub struct InputState {
  pub keyboard: KeyboardState,
  pub mouse: MouseState,
}

#[derive(Debug, Default)]
pub struct GameState {
  pub input: InputState,
}
