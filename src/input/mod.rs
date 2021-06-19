use winit::event::VirtualKeyCode;
use winit::dpi::PhysicalPosition;
use cgmath::Point3;

pub enum MyKeyboardInput {
  Key(Option<VirtualKeyCode>),
  CmdPressed(bool),
}

pub struct MyMouseInput {
  pub position: PhysicalPosition<f64>,
}


#[derive(Default)]
pub struct GameWantsExitEvent {}

pub struct CameraEnteredEvent {
  pub position: Point3<f32>,
}

pub enum InputEvent {
  KeyBoard(MyKeyboardInput),
  Mouse(MyMouseInput),
}

pub enum GameEvent {
  Draw(),
  Camera(CameraEnteredEvent),
  Game(GameWantsExitEvent)
}