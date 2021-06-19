use crate::input::CameraEnteredEvent;
use crate::components::{CameraId, Acceleration, Position, Velocity};
use crate::input::{MyKeyboardInput, MyMouseInput, GameEvent, InputEvent};

use bevy_ecs::event::{EventReader, EventWriter};
use bevy_ecs::system::{Query};

use cgmath::{Angle, InnerSpace, Rad, Vector3};
use winit::event::VirtualKeyCode;

pub fn camera_reacts_to_mouse_movement(mut mouse_reader: EventReader<InputEvent>, mut query: Query<&mut CameraId>) {
  for event in mouse_reader.iter() {
    match event {
      InputEvent::Mouse(event) => {
        for mut camera in query.iter_mut() {
          if let Some(lx) = camera.last_x {
            if let Some(ly) = camera.last_y {
              let mut xoffset: f32 = (event.position.x - lx) as f32;
              let mut yoffset: f32 = (ly - event.position.y) as f32; // reversed since y-coordinates range from bottom to top
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
          camera.last_x = Some(event.position.x);
          camera.last_y = Some(event.position.y);
        }
      }
      _ => {}
    }
  }
}


pub fn camera_reacts_to_keyboard(mut keyboard_reader: EventReader<InputEvent>, mut writer: EventWriter<GameEvent>, mut query: Query<(&mut Position, &mut CameraId)>) {
  for event in keyboard_reader.iter() {
    match event {
      InputEvent::KeyBoard(MyKeyboardInput::Key(Some(key_code))) => {
        for (mut position, mut camera) in query.iter_mut() {
          let mut moved = false;
          //let camera_speed = velocity.vec3;
          let zz = camera.front.cross(camera.up).normalize();
          match key_code {
            VirtualKeyCode::A => {
              moved = true;
              position.point3 += zz * camera.speed;
            }
            VirtualKeyCode::D => {
              moved = true;
              position.point3 += -zz * camera.speed;
            }
            VirtualKeyCode::W => {
              moved = true;
              position.point3 += camera.speed * camera.front;
            }
            VirtualKeyCode::S => {
              moved = true;
              position.point3 += -camera.speed * camera.front;
            }
            VirtualKeyCode::Q => {
              camera.yaw -= camera.speed;
            }
            VirtualKeyCode::E => {
              camera.yaw += camera.speed;
            }
            VirtualKeyCode::Z => {
              camera.pitch -= camera.speed;
            }
            VirtualKeyCode::C => {
              camera.pitch += camera.speed;
            }
            _ => {}
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
          } else {
          }
        }
      }
      _ => {}
    }
  }
}