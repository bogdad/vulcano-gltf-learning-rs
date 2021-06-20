use bevy_ecs::{system::Query, system::Res};
use cgmath::{Array, Vector3};

use crate::components::{CameraId, GameMode, GameState, Velocity};

pub fn camera_has_speed(game_state: Res<GameState>, mut query: Query<(&CameraId, &mut Velocity)>) {
  for (_camera_id, mut velocity) in query.iter_mut() {
    velocity.vec3 = if game_state.mode == GameMode::Play {
      Vector3::new(0.05, 0.0, 0.05)
    } else {
      Vector3::from_value(0.0)
    };
  }
}
