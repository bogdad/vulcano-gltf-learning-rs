use crate::input::InputEvent;
use crate::input::GameWantsExitEvent;
use bevy_ecs::system::Query;
use bevy_ecs::event::{EventReader, EventWriter};

use crate::components::{Position, Velocity, Acceleration, GameState};
use crate::input::{MyKeyboardInput, GameEvent};
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

pub fn game_reacts_to_keyboard(mut keyboard_reader: EventReader<InputEvent>, mut query: Query<&mut GameState>, mut event_writer: EventWriter<GameEvent>) {
  for mut game_state in query.iter_mut() {
    for event in keyboard_reader.iter() {
      match event {
        InputEvent::KeyBoard(MyKeyboardInput::Key(Some(key_code))) => {
          match key_code {
            VirtualKeyCode::Q => {
              if game_state.cmd_pressed {
                event_writer.send(GameEvent::Game(GameWantsExitEvent{}));
              }
            }
            _ => {}
          }
        }
        InputEvent::KeyBoard(MyKeyboardInput::CmdPressed(pressed)) => {
          game_state.cmd_pressed = *pressed;
        }
        _ => {}
      }
    }
  }
}