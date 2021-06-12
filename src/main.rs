use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::Format;
use vulkano::render_pass::RenderPass;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::{PhysicalDevice, PhysicalDeviceType};
use vulkano::instance::Instance;

use vulkano::swapchain::{
  ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform, Swapchain,
  SwapchainCreationError,
};

use vulkano_text::DrawText;
use vulkano_win::VkSurfaceBuild;

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};


extern crate futures;
extern crate itertools;
extern crate mint;
extern crate vulkano_text;
extern crate profiling;


use futures::executor::ThreadPoolBuilder;

use std::sync::Arc;

mod actor;
mod camera;
mod game;
mod sign_post;
mod sky;
mod sounds;
mod world;

mod executor;
mod render;
mod settings;
mod shaders;
mod things;
mod utils;


use executor::Executor;
use game::Game;
use render::model::Model;
use settings::Settings;
use shaders::{main, skybox};

pub enum GameEvent {
  Frame,
}

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
  fn new(event_loop: &EventLoop<GameEvent>) -> Graph {
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

      Swapchain::start(
        device.clone(),
        surface.clone())
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
    let (new_swapchain, new_images) = match self.swapchain.recreate().dimensions(dimensions).build() {
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
  thread_pool_builder.name_prefix("background").pool_size(2);
  let thread_pool = thread_pool_builder.create().unwrap();

  let event_loop = EventLoop::<GameEvent>::with_user_event();
  let graph = Graph::new(&event_loop);

  /*let dynamic_state = DynamicState {
      line_width: None,
      viewports: None,
      scissors: None,
      compare_mask: None,
      write_mask: None,
      reference: None,
  };*/

  let executor = Executor::new(thread_pool);

  let settings = Settings {
    sky_enabled: true,
    box_enabled: true,
    dog_enabled: true,
    letters_enabled: true,
    triangle_enabled: true,
    lap_enabled: true,
  };

  let mut game = Game::new(settings, executor, graph, &event_loop);

  game.init();
  event_loop.run(move |event, _, mut control_flow| {
    profiling::scope!("event_loop");
    game.tick();
    let res = game.gloop(event, &mut control_flow);
    profiling::finish_frame!();
    res
  });
}
