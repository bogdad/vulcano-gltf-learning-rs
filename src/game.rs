use cgmath::{Point3, Vector3};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SubpassContents};
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_text::DrawTextTrait;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use std::boxed::Box;
use std::path::Path;

use crate::camera::Camera;
use crate::executor::Executor;
use crate::render::system::System;
use crate::render::textures::Textures;
use crate::sign_post::SignPost;
use crate::things::lap::Lap;
use crate::things::primitives::{PrimitiveCube, PrimitiveTriangle};
use crate::things::texts::Texts;
use crate::world::World;
use crate::sounds::Sounds;
use crate::Graph;
use crate::Model;

pub struct Game {
  graph: Graph,
  camera: Camera,
  world: World,
  sounds: Sounds,
  recreate_swapchain: bool,
  models: Vec<Model>,
  previous_frame_end: Option<Box<dyn GpuFuture>>,
  system: System,
  cmd_pressed: bool,
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
      last_x: None,
      last_y: None,
      yaw: 0.0,
      pitch: 0.0,
    };

    let strs = (-200..200).map(|i| i.to_string()).collect();
    let texts = Texts::build(strs);

    let mut sign_posts = vec![];
    for i in -200..200 {
      sign_posts.push(SignPost::new(
        &graph.device,
        Point3::new(i as f32, -2.0, 0.0),
        i.to_string(),
        &texts,
      ));
    }

    for i in -200..200 {
      sign_posts.push(SignPost::new(
        &graph.device,
        Point3::new(-2.0, i as f32, 0.0),
        i.to_string(),
        &texts,
      ));
    }

    for i in -200..200 {
      sign_posts.push(SignPost::new(
        &graph.device,
        Point3::new(-2.0, -2.0, i as f32),
        i.to_string(),
        &texts,
      ));
    }

    let world = World::new(executor, &graph, sign_posts);

    let sounds = Sounds::new();

    let recreate_swapchain = false;

    let models = vec![
      //Model::from_gltf(Path::new("models/creature.glb"), &device),
      //Model::from_gltf(Path::new("models/creature2.glb"), &device),
      //Model::from_gltf(Path::new("models/creature3.glb"), &device),
      //Model::from_gltf(Path::new("models/landscape.glb"), &graph.device),
      Model::from_gltf(Path::new("models/dog.glb"), &graph.device),
      //Model::from_gltf(Path::new("models/box.glb"), &device),
      //Model::from_gltf(Path::new("models/center.glb"), &device),
      PrimitiveCube::new(2.0, 4.0, 8.0, (-8.0, 0.0, 0.0))
        .mesh
        .get_buffers(&graph.device),
      PrimitiveTriangle::new(Point3::new(10.0, 0.0, 0.0))
        .mesh
        .get_buffers(&graph.device),
      Lap::new(&graph.device).model,
    ];

    let textures = Textures::new(&texts);

    let (system, system_future) = System::new(&graph, textures);

    let previous_frame_end = Some(system_future);

    Game {
      graph,
      camera,
      world,
      recreate_swapchain,
      models,
      sounds,
      system,
      previous_frame_end,
      cmd_pressed: false,
    }
  }

  fn draw(&mut self) {
    self.previous_frame_end.as_mut().unwrap().cleanup_finished();
    if self.recreate_swapchain {
      self.graph.recreate_swapchain();
      self.system.recreate_swapchain(&self.graph);
      self.recreate_swapchain = false;
    }

    let set = self.system.main_set(
      self.camera.proj(&self.graph),
      &self.world.get_models(),
      self.camera.pos,
    );
    let set_skybox = self.system.skybox_set(self.camera.proj_skybox(&self.graph));

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
        self.system.framebuffers[image_num].clone(),
        SubpassContents::Inline,
        vec![
          [0.0, 0.0, 0.0, 1.0].into(),
          1f32.into(),
          [0.0, 0.0, 0.0, 1.0].into(),
          1f32.into(),
        ],
      )
      .unwrap();
    for model in &self.models {
      model.draw_indexed(&mut builder, self.system.pipeline.clone(), set.clone());
    }
    for model in self.world.get_models() {
      model
        .0
        .draw_indexed(&mut builder, self.system.pipeline.clone(), set.clone());
    }
    builder.next_subpass(SubpassContents::Inline).unwrap();
    for model in self.world.get_models_skybox() {
      model.0.draw_indexed(
        &mut builder,
        self.system.pipeline_skybox.clone(),
        set_skybox.clone(),
      );
    }
    builder.end_render_pass().unwrap();

    let mut y = 50.0;
    let status = self.status_string();
    for line in status.split('\n') {
      self
        .graph
        .draw_text
        .queue_text(200.0, y, 40.0, [1.0, 1.0, 1.0, 1.0], line);
      y += 40.0;
    }
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

  pub fn init(&mut self) {
    self.sounds.play();
  }

  pub fn tick(&mut self) {
    self.world.tick();
  }

  pub fn gloop(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
    match event {
      Event::WindowEvent {
        event: WindowEvent::ModifiersChanged(modifiers),
        ..
      } => {
        self.cmd_pressed = modifiers.logo();
      }
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
        if let KeyboardInput {
          virtual_keycode: Some(VirtualKeyCode::Q),
          ..
        } = input
        {
          if self.cmd_pressed {
            *control_flow = ControlFlow::Exit;
          }
        }
      }
      Event::WindowEvent {
        event: WindowEvent::CursorMoved { position, .. },
        ..
      } => {
        self.camera.react_mouse(&position);
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
