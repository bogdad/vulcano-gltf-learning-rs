use crate::input::CameraEnteredEvent;
use crate::components::*;
use crate::input::{GameEvent};

use bevy_ecs::event::{EventWriter};
use bevy_ecs::system::Res;
use bevy_ecs::system::{Query};

use cgmath::{Angle, InnerSpace, Rad, Vector3};

pub fn camera_reacts_to_input(input_state: Res<InputState>, mut writer: EventWriter<GameEvent>, mut query: Query<(&mut Position, &mut CameraId)>) {
  for (mut position, mut camera) in query.iter_mut() {
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
  }
}