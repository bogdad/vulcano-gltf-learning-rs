use crate::input::{GameEvent, MyKeyStatus};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::Format;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::instance::{PhysicalDevice, PhysicalDeviceType};
use vulkano::render_pass::RenderPass;

use vulkano::swapchain::{Surface, Swapchain, SwapchainCreationError};

use vulkano_text::DrawText;
use vulkano_win::VkSurfaceBuild;

use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

extern crate futures;
extern crate itertools;
extern crate mint;
extern crate profiling;
extern crate vulkano_text;

use futures::executor::ThreadPoolBuilder;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;

mod actor;
mod camera;
mod game;
mod myworld;
mod sign_post;
mod sky;
mod sounds;

mod components;
mod ecs;
mod executor;
mod input;
mod render;
mod settings;
mod shaders;
mod systems;
mod things;
mod utils;

use executor::Executor;
use game::Game;
use input::{InputEvent, MyKeyboardInput, MyMouseInput, MyMouseWheel};
use render::Model;
use settings::Settings;
use shaders::{main, skybox};

pub struct Graph {
  surface: Arc<Surface<Window>>,
  dimensions: [u32; 2],
  device: Arc<Device>,
  queue: Arc<Queue>,
  swapchain: Arc<Swapchain<Window>>,
  images: Vec<Arc<SwapchainImage<Window>>>,
  render_pass: Arc<RenderPass>,
  vs: main::vs::Shader,
  fs: main::fs::Shader,
  skybox_vs: skybox::vs::Shader,
  skybox_fs: skybox::fs::Shader,
  draw_text: DrawText,
}

impl Graph {
  fn new(event_loop: &EventLoop<()>) -> Graph {
    let required_extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, &required_extensions, vec![]).unwrap();

    for device in PhysicalDevice::enumerate(&instance) {
      println!(
        "possible device: {} (type: {:?})",
        device.name(),
        device.ty()
      );
    }
    let device_ext = DeviceExtensions {
      khr_swapchain: true,
      ..DeviceExtensions::none()
    };

    let surface = WindowBuilder::new()
      .with_inner_size(PhysicalSize::new(2400.0, 1600.0))
      .build_vk_surface(&event_loop, instance.clone())
      .unwrap();
    let dimensions: [u32; 2] = surface.window().inner_size().into();
    let physical = PhysicalDevice::enumerate(&instance)
      .find(|device| device.ty() == PhysicalDeviceType::DiscreteGpu)
      .unwrap();
    println!(
      "Using device: {} (type: {:?})",
      physical.name(),
      physical.ty()
    );
    let queue_family = physical
      .queue_families()
      .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
      .unwrap();

    let (device, mut queues) = Device::new(
      physical,
      physical.supported_features(),
      &device_ext,
      [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();
    let queue = queues.next().unwrap();
    let (swapchain, images) = {
      let caps = surface.capabilities(physical).unwrap();
      let alpha = caps.supported_composite_alpha.iter().next().unwrap();
      let format = caps.supported_formats[0].0;
      let dimensions: [u32; 2] = surface.window().inner_size().into();

      Swapchain::start(device.clone(), surface.clone())
        .num_images(caps.min_image_count)
        .format(format)
        .dimensions(dimensions)
        .usage(ImageUsage::color_attachment())
        .sharing_mode(&queue)
        .composite_alpha(alpha)
        .build()
        .unwrap()
    };

    let render_pass = Arc::new(
      vulkano::ordered_passes_renderpass!(
          device.clone(),
          attachments: {
              final_color: {
                  load: Clear,
                  store: Store,
                  format: swapchain.format(),
                  samples: 1,
              },
              depth: {
                  load: Clear,
                  store: Store,
                  format: Format::D16Unorm,
                  samples: 1,
              },
              color: {
                  load: Clear,
                  store: Store,
                  format: swapchain.format(),
                  samples: 1,
              },
              depth2: {
                  load: Clear,
                  store: DontCare,
                  format: Format::D16Unorm,
                  samples: 1,
              }
          },
          passes: [
          {
              color: [color],
              depth_stencil: {depth},
              input: []
          },
          {
              color: [final_color],
              depth_stencil: {depth2},
              input: [color, depth]
          }
          ]
      )
      .unwrap(),
    );
    let vs = main::vs::Shader::load(device.clone()).unwrap();
    //let tcs = tcs::Shader::load(device.clone()).unwrap();
    //let tes = tes::Shader::load(device.clone()).unwrap();
    let fs = main::fs::Shader::load(device.clone()).unwrap();
    let skybox_vs = skybox::vs::Shader::load(device.clone()).unwrap();
    let skybox_fs = skybox::fs::Shader::load(device.clone()).unwrap();

    let draw_text = DrawText::new(device.clone(), queue.clone(), swapchain.clone(), &images);

    Graph {
      surface,
      dimensions,
      device,
      queue,
      swapchain,
      images,
      render_pass,
      vs,
      fs,
      skybox_vs,
      skybox_fs,
      draw_text,
    }
  }

  pub fn recreate_swapchain(&mut self) {
    let dimensions: [u32; 2] = self.surface.window().inner_size().into();
    let (new_swapchain, new_images) = match self.swapchain.recreate().dimensions(dimensions).build()
    {
      Ok(r) => r,
      Err(SwapchainCreationError::UnsupportedDimensions) => return,
      Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
    };
    self.swapchain = new_swapchain;
    self.images = new_images.clone();

    self.draw_text = DrawText::new(
      self.device.clone(),
      self.queue.clone(),
      self.swapchain.clone(),
      &new_images,
    );
  }
}

fn main() {
  let mut thread_pool_builder = ThreadPoolBuilder::new();
  thread_pool_builder
    .name_prefix("background")
    .pool_size(3)
    .after_start(|i| {
      profiling::register_thread!();
    });
  let thread_pool = thread_pool_builder.create().unwrap();

  let event_loop = EventLoop::<()>::new();
  let graph = Graph::new(&event_loop);

  let executor = Executor::new(thread_pool);

  let settings = Settings {
    sky_enabled: true,
    box_enabled: true,
    dog_enabled: true,
    letters_enabled: true,
    triangle_enabled: true,
    lap_enabled: true,
  };

  let (send, recv) = channel();

  let game_exited = Arc::new(AtomicBool::new(false));
  let game_exited_clone = Arc::clone(&&game_exited);
  let thread_handle = Arc::new(Mutex::new(Some(
    std::thread::Builder::new()
      .name(format!("gameloop"))
      .spawn(move || {
        let mut game = Game::new(settings, executor, graph, game_exited_clone, recv);
        game.game_loop();
      })
      .unwrap(),
  )));
  let thread_handle_clone = Arc::clone(&thread_handle);

  event_loop.run(move |event, _, control_flow| {
    profiling::scope!("event_loop");
    if game_exited.load(Ordering::Acquire) {
      println!("exiting..");
      *control_flow = ControlFlow::Exit;
    }
    match event {
      Event::WindowEvent {
        event: WindowEvent::ModifiersChanged(modifiers),
        ..
      } => send
        .send(InputEvent::KeyBoard(MyKeyboardInput::CmdPressed(
          modifiers.logo(),
        )))
        .unwrap(),
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => {
        game_exited.store(true, Ordering::Release);
        let mut thread_handle = thread_handle_clone.lock().unwrap();
        thread_handle.take().unwrap().join().unwrap();
        *control_flow = ControlFlow::Exit;
      }
      Event::WindowEvent {
        event: WindowEvent::Resized(_),
        ..
      } => send.send(InputEvent::RecreateSwapchain {}).unwrap(),
      Event::WindowEvent {
        event: WindowEvent::KeyboardInput { input, .. },
        ..
      } => {
        let status = match input.state {
          ElementState::Released => MyKeyStatus::Released,
          ElementState::Pressed => MyKeyStatus::Pressed,
        };
        send
          .send(InputEvent::KeyBoard(MyKeyboardInput::Key {
            key_code: input.virtual_keycode,
            status,
          }))
          .unwrap();
      }
      Event::WindowEvent {
        event: WindowEvent::MouseWheel { delta, .. },
        ..
      } => send
        .send(InputEvent::MouseWheel(MyMouseWheel { delta }))
        .unwrap(),
      Event::WindowEvent {
        event: WindowEvent::CursorMoved { position, .. },
        ..
      } => send
        .send(InputEvent::MouseMoved(MyMouseInput { position }))
        .unwrap(),
      _ => (),
    }
  });
}
