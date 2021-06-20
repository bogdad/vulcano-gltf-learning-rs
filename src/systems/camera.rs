use crate::components::*;

use crate::input::CameraEnteredEvent;
use crate::input::GameEvent;

use bevy_ecs::change_detection::Mut;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;

use cgmath::{Angle, InnerSpace, Rad, Vector3};

fn react_to_keyboard(
  game_state: &Res<GameState>,
  mut writer: &mut EventWriter<GameEvent>,
  mut position: &mut Mut<Position>,
  mut camera: &mut Mut<CameraId>,
) {
  if game_state.mode == GameMode::Edit {
    let input_state = &game_state.input;
    let mut moved = false;
    //let camera_speed = velocity.vec3;
    let zz = camera.front.cross(camera.up).normalize();
    if input_state.keyboard.a {
      moved = true;
      position.point3 += zz * camera.speed;
    }
    if input_state.keyboard.d {
      moved = true;
      position.point3 += -zz * camera.speed;
    }
    if input_state.keyboard.w {
      moved = true;
      position.point3 += camera.speed * camera.front;
    }
    if input_state.keyboard.s {
      moved = true;
      position.point3 += -camera.speed * camera.front;
    }
    if input_state.keyboard.q {
      camera.yaw -= camera.speed;
    }
    if input_state.keyboard.e {
      camera.yaw += camera.speed;
    }
    if input_state.keyboard.z {
      camera.pitch -= camera.speed;
    }
    if input_state.keyboard.c {
      camera.pitch += camera.speed;
    }
    let direction = Vector3::new(
      Rad(camera.yaw).cos() * Rad(camera.pitch).cos(),
      Rad(camera.pitch).sin(),
      Rad(camera.yaw).sin() * Rad(camera.pitch).cos(),
    );
    camera.front = direction.normalize();
    if moved {
      writer.send(GameEvent::Camera(CameraEnteredEvent {
        position: position.point3,
      }))
    }
  } else {
  }
}

fn react_to_mouse(
  game_state: &Res<GameState>,
  mut position: &mut Mut<Position>,
  mut camera: &mut Mut<CameraId>,
) {
  let input_state = &game_state.input;
  if let Some(lx) = camera.last_x {
    if let Some(ly) = camera.last_y {
      let mut xoffset: f32 = input_state.mouse.position.x - lx;
      let mut yoffset: f32 = ly - input_state.mouse.position.y; // reversed since y-coordinates range from bottom to top
      let sensitivity = 0.1;
      xoffset *= sensitivity;
      yoffset *= sensitivity;
      camera.yaw += xoffset;
      camera.pitch += yoffset;
      let direction = Vector3::new(
        Rad(camera.yaw).cos() * Rad(camera.pitch).cos(),
        Rad(camera.pitch).sin(),
        Rad(camera.yaw).sin() * Rad(camera.pitch).cos(),
      );
      camera.front = direction.normalize();
    }
  }
  camera.last_x = Some(input_state.mouse.position.x);
  camera.last_y = Some(input_state.mouse.position.y);
  if let Some(last_wheel_x) = camera.last_wheel_x {
    if let Some(last_wheel_y) = camera.last_wheel_y {
      let diff_y = input_state.mouse.scroll_position.y - last_wheel_y;
      let diff_x = input_state.mouse.scroll_position.x - last_wheel_x;
      position.point3.x += diff_x;
      position.point3.y += diff_y;
    }
  }
  camera.last_wheel_x = Some(input_state.mouse.scroll_position.x);
  camera.last_wheel_y = Some(input_state.mouse.scroll_position.y);
}

pub fn camera_reacts_to_input(
  game_state: Res<GameState>,
  mut writer: EventWriter<GameEvent>,
  mut query: Query<(&mut Position, &mut CameraId)>,
) {
  let input_state = &game_state.input;
  let mode = &game_state.mode;
  for (mut position, mut camera) in query.iter_mut() {
    react_to_keyboard(&game_state, &mut writer, &mut position, &mut camera);
    react_to_mouse(&game_state, &mut position, &mut camera);
  }
}
