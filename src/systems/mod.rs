use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::*;
use crate::components::{Position, Velocity, CameraId, CameraEnteredEvent};
use crate::input::{MyKeyboardInput, MyMouseInput};
use cgmath::{Vector3, Rad, Angle, InnerSpace};
use winit::event::VirtualKeyCode;


pub fn camera_reacts_to_mouse_movement(mut mouse_reader: EventReader<MyMouseInput>, mut query: Query<(&mut Velocity, &mut CameraId)>) {
  for event in mouse_reader.iter() {
    for (mut velocity, mut camera) in query.iter_mut() {
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
}

// This system moves each entity with a Position and Velocity component
pub fn movement(mut query: Query<(&mut Position, &Velocity)>) {
  for (mut position, velocity) in query.iter_mut() {
      position.point3 += velocity.vec3;
  }
}

pub fn camera_reacts_to_keyboard(mut keyboard_reader: EventReader<MyKeyboardInput>, mut writer: EventWriter<CameraEnteredEvent>, mut query: Query<(&mut Velocity, &mut CameraId, &Position)>) {
  for event in keyboard_reader.iter() {
    match event {
      MyKeyboardInput::Key(Some(key_code)) => {
        for (mut velocity, mut camera, position) in query.iter_mut() {
          let mut moved = false;
          //let camera_speed = velocity.vec3;
          let zz = camera.front.cross(camera.up).normalize();
          match key_code {
            VirtualKeyCode::A => {
              moved = true;
              velocity.vec3 = zz * camera.speed;
            }
            VirtualKeyCode::D => {
              moved = true;
              velocity.vec3 = -zz * camera.speed;
            }
            VirtualKeyCode::W => {
              moved = true;
              velocity.vec3 = camera.speed * camera.front;
            }
            VirtualKeyCode::S => {
              moved = true;
              velocity.vec3 = -camera.speed * camera.front;
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
            writer.send(CameraEnteredEvent {
              position: position.point3,
            })
          }
        }
      }
      _ => {}
    }
  }
}