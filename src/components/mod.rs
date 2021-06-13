use cgmath::{Vector3, Point3, Array};
use bevy_ecs::bundle::Bundle;

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
  pub last_x: Option<f64>,
  pub last_y: Option<f64>,
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
    }
  }
}

#[derive(Bundle, Default)]
pub struct CameraBundle {
    pub camera: CameraId,
    pub position: Position,
    pub velocity: Velocity,
}

pub struct CameraEnteredEvent {
  pub position: Point3<f32>,
}