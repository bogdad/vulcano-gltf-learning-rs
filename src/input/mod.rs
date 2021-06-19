use winit::event::VirtualKeyCode;
use winit::dpi::PhysicalPosition;
use cgmath::Point3;

#[derive(Debug)]
pub enum MyKeyStatus {
  Pressed,
  Released,
}

#[derive(Debug)]
pub enum MyKeyboardInput {
  Key{key_code:Option<VirtualKeyCode>, status: MyKeyStatus},
  CmdPressed(bool),
}

#[derive(Debug)]
pub struct MyMouseInput {
  pub position: PhysicalPosition<f64>,
}


#[derive(Default)]
pub struct GameWantsExitEvent {}

pub struct CameraEnteredEvent {
  pub position: Point3<f32>,
}

#[derive(Debug)]
pub enum InputEvent {
  KeyBoard(MyKeyboardInput),
  Mouse(MyMouseInput),
}

pub enum GameEvent {
  Draw(),
  Camera(CameraEnteredEvent),
  Game(GameWantsExitEvent)
}