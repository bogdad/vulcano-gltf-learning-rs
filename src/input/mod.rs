use winit::event::VirtualKeyCode;
use winit::dpi::PhysicalPosition;

pub enum MyKeyboardInput {
  Key(Option<VirtualKeyCode>),
  CmdPressed(bool),
}

pub struct MyMouseInput {
  pub position: PhysicalPosition<f64>,
}