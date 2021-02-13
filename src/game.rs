use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::{AutoCommandBufferBuilder, SubpassContents};
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet, PersistentDescriptorSetBuf};
use vulkano::image::ImmutableImage;
use vulkano::format::Format;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_text::{DrawTextTrait};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use cgmath::{Point3, Vector3};

use std::sync::Arc;

use crate::vs;
use crate::executor::Executor;
use crate::Graph;
use crate::Model;
use crate::camera::Camera;
use crate::world::World;
use crate::things::primitives::{PrimitiveCube, PrimitiveTriangle};
use crate::things::texts::Texts;
use crate::render::textures::Textures;
use crate::sign_post::SignPost;

pub struct Game {
  executor: Executor,
  graph: Graph,
  camera: Camera,
  world: World,
  recreate_swapchain: bool,
  models: Vec<Model>,
  uniform_buffer: CpuBufferPool<vs::ty::Data>,
  previous_frame_end: Option<Box<dyn GpuFuture>>,
  texts: Texts,
  textures: Textures,
  sampler: Option<Arc<Sampler>>,
  texture: Option<Arc<ImmutableImage<Format>>>
}

impl Game {
  pub fn new(executor: Executor, graph: Graph) -> Game {
    // gltf:
    // "and the default camera sits on the
    // -Z side looking toward the origin with +Y up"
    //                               x     y    z
    // y = up/down
    // x = left/right
    // z = close/far
    let camera = Camera {
      pos: Point3::new(0.0, -1.0, -1.0),
      front: Vector3::new(0.0, 0.0, 1.0),
      up: Vector3::new(0.0, 1.0, 0.0),
      speed: 0.3,
    };

    let sign_posts = vec![
      SignPost::new(&graph.device, Point3::new(-10.0, 0.0, 0.0), "100".to_string())
    ];

    let world = World::new(executor.clone(), &graph, sign_posts);

    let recreate_swapchain = false;


    let models = vec![
      //Model::from_gltf(Path::new("models/creature.glb"), &device),
      //Model::from_gltf(Path::new("models/creature2.glb"), &device),
      //Model::from_gltf(Path::new("models/creature3.glb"), &device),
      //Model::from_gltf(Path::new("models/landscape.glb"), &graph.device),
      //Model::from_gltf(Path::new("models/dog.glb"), &graph.device),
      //Model::from_gltf(Path::new("models/box.glb"), &device),
      //Model::from_gltf(Path::new("models/center.glb"), &device),
      PrimitiveCube::new(2.0, 4.0, 8.0, (-4.0, 0.0, 0.0)).mesh.get_buffers(&graph.device),
      PrimitiveTriangle::new(Point3::new(0.0, 0.0, 0.0)).mesh.get_buffers(&graph.device),
    ];

    let uniform_buffer =
      CpuBufferPool::<vs::ty::Data>::new(graph.device.clone(), BufferUsage::all());

    let strs = (-100..100).map(|i| i.to_string()).collect();
    let texts = Texts::build(strs);

    let textures = Textures::new(&texts);

    let previous_frame_end = Some(sync::now(graph.device.clone()).boxed());

    Game {
      executor,
      graph,
      camera,
      world,
      recreate_swapchain,
      models,
      uniform_buffer,
      previous_frame_end,
      texts,
      textures,
      sampler: None,
      texture: None,
    }
  }

  pub fn init(&mut self) {
    let (texture, future) = self.textures.draw(&self.graph.queue);
    self.previous_frame_end = Some(future);

    let sampler = Sampler::new(
        self.graph.device.clone(),
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0,
        1.0,
        0.0,
        0.0,
    )
    .unwrap();
    self.sampler = Some(sampler);
    self.texture = Some(texture);
  }

  fn draw(&mut self) {
    self.previous_frame_end.as_mut().unwrap().cleanup_finished();
    if self.recreate_swapchain {
      self.graph.recreate_swapchain();
      self.recreate_swapchain = false;
    }
    let uniform_buffer_subbuffer = {

      let uniform_data = self.camera.proj(&self.graph);
      self.uniform_buffer.next(uniform_data).unwrap()
    };
    let layout = self.graph.pipeline.descriptor_set_layout(0).unwrap();
    let set = Arc::new(
      PersistentDescriptorSet::start(layout.clone())
        .add_buffer(uniform_buffer_subbuffer)
        .unwrap()
        .add_sampled_image(self.texture.as_ref().unwrap().clone(), self.sampler.as_ref().unwrap().clone())
        .unwrap()
        .build()
        .unwrap(),
    );

    let (image_num, suboptimal, acquire_future) =
      match swapchain::acquire_next_image(self.graph.swapchain.clone(), None) {
        Ok(r) => r,
        Err(AcquireError::OutOfDate) => {
          self.recreate_swapchain = true;
          return;
        }
        Err(e) => panic!("Failed to acquire next image: {:?}", e),
      };

    if suboptimal {
      self.recreate_swapchain = true;
    }

    let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
      self.graph.device.clone(),
      self.graph.queue.family(),
    )
    .unwrap();
    builder
      .begin_render_pass(
        self.graph.framebuffers[image_num].clone(),
        SubpassContents::Inline,
        vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()],
      )
      .unwrap();
    for model in &self.models {
      model.draw_indexed(&mut builder, self.graph.pipeline.clone(), set.clone())
    }
    for model in self.world.get_models() {
      model.draw_indexed(&mut builder, self.graph.pipeline.clone(), set.clone());
    }


    let mut y = 50.0;
    let status = self.status_string();
    for line in status .split("\n") {
      self.graph.draw_text.queue_text(200.0, y, 40.0, [1.0, 1.0, 1.0, 1.0], line);
      y += 40.0;
    };

    builder.end_render_pass().unwrap();
    builder.draw_text(&mut self.graph.draw_text, image_num);

    let command_buffer = builder.build().unwrap();

    let future = self
      .previous_frame_end
      .take()
      .unwrap()
      .join(acquire_future)
      .then_execute(self.graph.queue.clone(), command_buffer)
      .unwrap()
      .then_swapchain_present(
        self.graph.queue.clone(),
        self.graph.swapchain.clone(),
        image_num,
      )
      .then_signal_fence_and_flush();

    match future {
      Ok(future) => {
        self.previous_frame_end = Some(future.boxed());
      }
      Err(FlushError::OutOfDate) => {
        self.recreate_swapchain = true;
        self.previous_frame_end = Some(sync::now(self.graph.device.clone()).boxed());
      }
      Err(e) => {
        println!("Failed to flush future: {:?}", e);
        self.previous_frame_end = Some(sync::now(self.graph.device.clone()).boxed());
      }
    }
  }

  pub fn tick(&mut self) {
    self.world.tick();
  }

  pub fn gloop(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => {
        *control_flow = ControlFlow::Exit;
      }
      Event::WindowEvent {
        event: WindowEvent::Resized(_),
        ..
      } => {
        self.recreate_swapchain = true;
      }
      Event::WindowEvent {
        event: WindowEvent::KeyboardInput { input, .. },
        ..
      } => {
        self.world.react(&input);
        let camera_moved = self.camera.react(self.world.mode, &input);
        if camera_moved {
          self.world.camera_entered(&self.camera.pos);
        }
      }
      Event::RedrawEventsCleared => {
        self.draw();
      }
      _ => (),
    }
  }

  fn status_string(&self) -> String {
    format!("world {}\ncamera {}", self.world, self.camera)
  }
}
