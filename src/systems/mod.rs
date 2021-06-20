use crate::input::GameWantsExitEvent;
use crate::input::InputEvent;
use crate::input::MyMouseWheel;
use bevy_ecs::event::{EventReader, EventWriter};
use bevy_ecs::system::Query;
use bevy_ecs::system::{Res, ResMut};
use cgmath::Vector1;
use cgmath::Vector2;
use winit::event::MouseScrollDelta;

use crate::components::*;
use crate::input::{GameEvent, MyKeyStatus, MyKeyboardInput, MyMouseInput};
use cgmath::Point2;
use winit::event::VirtualKeyCode;

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

pub fn game_reacts_to_keyboard(
  mut game_state: ResMut<GameState>,
  mut event_writer: EventWriter<GameEvent>,
) {
  let input_state = &game_state.input;
  if input_state.keyboard.cmd && input_state.keyboard.q {
    event_writer.send(GameEvent::Game(GameWantsExitEvent {}));
  }
  if input_state.keyboard.esc {
    game_state.mode = match game_state.mode {
      GameMode::Edit => GameMode::Play,
      GameMode::Play => GameMode::Edit,
    }
  }
}

pub fn input_state_from_game_events(
  mut keyboard_reader: EventReader<InputEvent>,
  mut game_state: ResMut<GameState>,
) {
  let mut input = &mut game_state.input;
  for event in keyboard_reader.iter() {
    match event {
      InputEvent::MouseWheel(MyMouseWheel { delta }) => {
        input.mouse.scroll_position = match delta {
          MouseScrollDelta::LineDelta(x, y) => input.mouse.scroll_position + Vector2::new(*x, *y),
          MouseScrollDelta::PixelDelta(pos) => {
            input.mouse.scroll_position + Vector2::new(pos.x as f32, pos.y as f32)
          }
        }
      }
      InputEvent::MouseMoved(MyMouseInput { position }) => {
        input.mouse.position = Point2::new(position.x as f32, position.y as f32);
      }
      InputEvent::KeyBoard(MyKeyboardInput::CmdPressed(pressed)) => {
        input.keyboard.cmd = *pressed;
      }
      InputEvent::KeyBoard(MyKeyboardInput::Key {
        key_code: Some(key_code),
        status,
      }) => match key_code {
        VirtualKeyCode::Escape => {
          input.keyboard.esc = match status {
            MyKeyStatus::Released => false,
            MyKeyStatus::Pressed => true,
          };
        }
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
      },
      _ => {}
    }
  }
}
