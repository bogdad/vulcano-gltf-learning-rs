use cgmath::Point3;
use winit::dpi::PhysicalPosition;
use winit::event::{MouseScrollDelta, VirtualKeyCode};

#[derive(Debug)]
pub enum MyKeyStatus {
  Pressed,
  Released,
}

#[derive(Debug)]
pub enum MyKeyboardInput {
  Key {
    key_code: Option<VirtualKeyCode>,
    status: MyKeyStatus,
  },
  CmdPressed(bool),
}

#[derive(Debug)]
pub struct MyMouseInput {
  pub position: PhysicalPosition<f64>,
}

#[derive(Debug)]
pub struct MyMouseWheel {
  pub delta: MouseScrollDelta,
}

#[derive(Default)]
pub struct GameWantsExitEvent {}

pub struct CameraEnteredEvent {
  pub position: Point3<f32>,
}

#[derive(Debug)]
pub enum InputEvent {
  KeyBoard(MyKeyboardInput),
  MouseMoved(MyMouseInput),
  MouseWheel(MyMouseWheel),
}

pub enum GameEvent {
  Draw(),
  Camera(CameraEnteredEvent),
  Game(GameWantsExitEvent),
}
