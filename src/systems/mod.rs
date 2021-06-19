use crate::input::InputEvent;
use crate::input::GameWantsExitEvent;
use bevy_ecs::system::Query;
use bevy_ecs::system::{Res, ResMut};
use bevy_ecs::event::{EventReader, EventWriter};

use crate::components::*;
use crate::input::{MyKeyboardInput, GameEvent, MyKeyStatus, MyMouseInput};
use winit::event::VirtualKeyCode;
use cgmath::Point2;

mod camera;

pub use camera::*;


// This system moves each entity with a Position and Velocity component
pub fn movement(mut query: Query<(&mut Position, &Velocity)>) {
  for (mut position, velocity) in query.iter_mut() {
      position.point3 += velocity.vec3;
  }
}

pub fn velocity_accel(mut query: Query<(&mut Velocity, &Acceleration)>) {
  for (mut velocity, accel) in query.iter_mut() {
    velocity.vec3 += accel.vec3;
  }
}

pub fn game_reacts_to_keyboard(input_state: Res<InputState>, mut event_writer: EventWriter<GameEvent>) {
  if input_state.keyboard.cmd && input_state.keyboard.q {
    event_writer.send(GameEvent::Game(GameWantsExitEvent{}));
  }
}

pub fn input_state_from_game_events(mut keyboard_reader: EventReader<InputEvent>, mut input: ResMut<InputState>) {
  for event in keyboard_reader.iter() {
    match event {
      InputEvent::Mouse(MyMouseInput{position}) => {
        input.mouse.position = Point2::new(position.x as f32, position.y as f32);
      }
      InputEvent::KeyBoard(MyKeyboardInput::CmdPressed(pressed)) => {
        input.keyboard.cmd = *pressed;
      }
      InputEvent::KeyBoard(MyKeyboardInput::Key { key_code: Some(key_code), status }) => {
        match key_code {
          VirtualKeyCode::Q => {
              input.keyboard.q = match status {
                MyKeyStatus::Released => false,
                MyKeyStatus::Pressed => true,
              };
          }
          VirtualKeyCode::W => {
            input.keyboard.w = match status {
              MyKeyStatus::Released => false,
              MyKeyStatus::Pressed => true,
            };
          }
          VirtualKeyCode::E => {
            input.keyboard.e = match status {
              MyKeyStatus::Released => false,
              MyKeyStatus::Pressed => true,
            };
          }
          VirtualKeyCode::A => {
            input.keyboard.a = match status {
              MyKeyStatus::Released => false,
              MyKeyStatus::Pressed => true,
            };
          }
          VirtualKeyCode::S => {
            input.keyboard.s = match status {
              MyKeyStatus::Released => false,
              MyKeyStatus::Pressed => true,
            };
          }
          VirtualKeyCode::D => {
            input.keyboard.d = match status {
              MyKeyStatus::Released => false,
              MyKeyStatus::Pressed => true,
            };
          }
          VirtualKeyCode::Z => {
            input.keyboard.z = match status {
              MyKeyStatus::Released => false,
              MyKeyStatus::Pressed => true,
            };
          }
          VirtualKeyCode::X => {
            input.keyboard.x = match status {
              MyKeyStatus::Released => false,
              MyKeyStatus::Pressed => true,
            };
          }
          VirtualKeyCode::C => {
            input.keyboard.c = match status {
              MyKeyStatus::Released => false,
              MyKeyStatus::Pressed => true,
            };
          }
          _ => {}
        }
      }
      _ => {}
    }
  }
}