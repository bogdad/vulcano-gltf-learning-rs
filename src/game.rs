use crate::input::InputEvent;
use crate::input::GameWantsExitEvent;
use crate::input::GameEvent;
use bevy_ecs::event::ManualEventReader;
use cgmath::{Point3};
use vulkano::command_buffer::{AutoCommandBufferBuilder, SubpassContents, CommandBufferUsage};
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_text::DrawTextTrait;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use profiling;

use bevy_ecs::event::{Events, EventReader};

use std::boxed::Box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Instant, Duration};
use std::path::Path;
use std::thread::JoinHandle;
use std::vec::Vec;

use crate::input::MyMouseInput;
use crate::input::MyKeyboardInput;
use crate::camera::Camera;
use crate::executor::Executor;
use crate::render::System;
use crate::render::Textures;
use crate::sign_post::SignPost;
use crate::sounds::Sounds;
use crate::things::Lap;
use crate::things::{PrimitiveCube, PrimitiveTriangle};
use crate::things::Texts;
use crate::things::Signal;
use crate::things::CountingWindowAvg;
use crate::myworld::MyWorld;
use crate::Graph;
use crate::Model;
use crate::Settings;
use crate::ecs::{Ecs};

pub struct Game {

  ecs: Ecs,
  events_reader: Option<ManualEventReader<GameEvent>>,

  camera: Camera,
  settings: Settings,
  graph: Graph,

  myworld: MyWorld,
  sounds: Sounds,
  recreate_swapchain: bool,
  models: Vec<Model>,
  previous_frame_end: Option<Box<dyn GpuFuture>>,
  i_frame: u64,
  last_frame_took: Arc<AtomicU32>,
  frame_signal: Arc<Signal>,
  system: System,
  cmd_pressed: bool,
  game_exited: Arc<AtomicBool>,
  ticker_thread: Option<JoinHandle<()>>,
  frame_times_avg: CountingWindowAvg,
}

impl Game {

  pub fn new(settings: Settings, executor: Executor, graph: Graph, event_loop: &EventLoop<GameEvent>) -> Game {

    let mut ecs = Ecs::new();

    let camera = Camera::new(&mut ecs);

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

    let myworld = MyWorld::new(settings.clone(), executor, &graph, sign_posts);

    let sounds = Sounds::new();

    let recreate_swapchain = false;

    let mut models = vec![];
    if settings.dog_enabled {
      models.push(Model::from_gltf(Path::new("models/dog.glb"), &graph.device));
    };
    if settings.box_enabled {
      models.push(
        PrimitiveCube::new(2.0, 4.0, 8.0, (-8.0, 0.0, 0.0))
          .mesh
          .get_buffers(&graph.device),
      );
    };
    if settings.box_enabled {
      models.push(
        PrimitiveTriangle::new(Point3::new(10.0, 0.0, 0.0))
          .mesh
          .get_buffers(&graph.device),
      );
    };
    if settings.lap_enabled {
      models.push(Lap::new(&graph.device).model);
    };

    let textures = Textures::new(&texts);

    let (system, system_future) = System::new(&graph, textures);

    let previous_frame_end = Some(system_future);

    let event_loop_proxy = event_loop.create_proxy();

    let game_exited = Arc::new(AtomicBool::new(false));
    let game_exited_local = Arc::clone(&game_exited);
    let last_frame_took = Arc::new(AtomicU32::new(0));
    let last_frame_took_clone = last_frame_took.clone();
    let frame_signal = Arc::new(Signal::new());
    let frame_signal_clone = frame_signal.clone();
    let ticker_thread = Some(std::thread::Builder::new()
    .name(format!("ticker"))
    .spawn(move ||  {
        while !game_exited_local.load(Ordering::Acquire) {
          let last_frame_took = last_frame_took_clone.load(Ordering::Acquire);
          // 1000 ms / 30 fps = 33 ms
          let last_frame_took_duration = Duration::from_millis(last_frame_took as u64);
          let interval = std::time::Duration::from_millis(33);
          if interval > last_frame_took_duration {
            let sleep = interval - last_frame_took_duration;
            std::thread::sleep(sleep);
          } else {
          }
          let result = event_loop_proxy.send_event(GameEvent::Draw());
          match result {
            Ok(()) => (),
            Err(_) => {
              break;
            }
          }
          let _ = frame_signal_clone.wait_and_reset();
        }
    }).unwrap());

    let frame_times_avg = CountingWindowAvg::new(30);

    Game {
      ecs,
      events_reader: None,
      settings,
      graph,
      camera,
      myworld,
      recreate_swapchain,
      models,
      sounds,
      system,
      previous_frame_end,
      i_frame: 0,
      last_frame_took,
      frame_signal,
      cmd_pressed: false,
      game_exited,
      ticker_thread,
      frame_times_avg,
    }
  }

  #[profiling::function]
  pub fn init(&mut self) {
    self.myworld.init(&self.ecs);
    self.sounds.play();
    let reader = self.ecs.get_events::<GameEvent>().get_reader();
    self.events_reader = Some(reader);
  }

  #[profiling::function]
  fn draw(&mut self) {
    let frame_start = Instant::now();
    self.i_frame = self.i_frame + 1;
    {
      profiling::scope!("cleanup_finished");
      if self.i_frame % 30 == 0 {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();
      }
    }
    if self.recreate_swapchain {
      profiling::scope!("recreate_swap_chain");
      self.graph.recreate_swapchain();
      self.system.recreate_swapchain(&self.graph);
      self.recreate_swapchain = false;
    }

    let set = {
      profiling::scope!("main_set");
      self.system.main_set(
      self.camera.proj(&self.graph, &self.ecs.world),
      self.myworld.get_scenes(),
      self.camera.get_pos(&self.ecs.world),
    )
     };

    let set_skybox = {
      profiling::scope!("sky_box_set");
      self.system.skybox_set(self.camera.proj_skybox(&self.graph, &self.ecs.world))
    };

    let (image_num, suboptimal, acquire_future) = {
      profiling::scope!("acquire_next_image");
      let (image_num, suboptimal, acquire_future) =
      match swapchain::acquire_next_image(self.graph.swapchain.clone(), None) {
        Ok(r) => r,
        Err(AcquireError::OutOfDate) => {
          self.recreate_swapchain = true;
          return;
        }
        Err(e) => panic!("Failed to acquire next image: {:?}", e),
      };
      (image_num, suboptimal, acquire_future)
    };

    if suboptimal {
      self.recreate_swapchain = true;
    }

    let mut builder = AutoCommandBufferBuilder::primary(
      self.graph.device.clone(),
      self.graph.queue.family(),
      CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    {
      profiling::scope!("begin-render-pass");
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
    }
    {
    profiling::scope!("iterate-models");
    for model in &self.models {
      model.draw_indexed(&mut builder, self.system.pipeline.clone(), set.clone());
    }
    }
    {
    profiling::scope!("iterate-myworld-models");
    for model in self.myworld.get_models() {
      model.draw_indexed(&mut builder, self.system.pipeline.clone(), set.clone());
    }
    }
    builder.next_subpass(SubpassContents::Inline).unwrap();
    {
      profiling::scope!("iterate-myworld-models");
    for model in self.myworld.get_models_skybox() {
      model.draw_indexed(
        &mut builder,
        self.system.pipeline_skybox.clone(),
        set_skybox.clone(),
      );
    }
    }
    builder.end_render_pass().unwrap();
    {
      profiling::scope!("draw-text");
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
    }
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
    let frame_end = Instant::now();
    let last_frame = (frame_end - frame_start).as_millis() as u32;
    self.last_frame_took.store(last_frame, Ordering::Release);
    self.frame_signal.signal();
    self.frame_times_avg.add(last_frame);
  }

  pub fn tick(&mut self) {
    self.myworld.tick(&self.ecs);
    self.ecs.tick();
  }

  #[profiling::function]
  pub fn gloop(&mut self, event: Event<GameEvent>, control_flow: &mut ControlFlow) {
    *control_flow = ControlFlow::Wait;
    let events = self.ecs.get_events::<GameEvent>();
    for event in self.events_reader.as_mut().unwrap().iter(events) {
      match event {
        GameEvent::Game(GameWantsExitEvent {}) => {
          self.game_exited.store(true, Ordering::Release);
          *control_flow = ControlFlow::Exit;
        }
        _ => {}
      }
    }
    match event {
      Event::UserEvent(game_event) => {
        match game_event {
          GameEvent::Draw() => {
            self.draw();
          }
          _ => (),
        }
      }
      Event::WindowEvent {
        event: WindowEvent::ModifiersChanged(modifiers),
        ..
      } => {
        self.ecs.get_events_mut::<InputEvent>().send(InputEvent::KeyBoard(MyKeyboardInput::CmdPressed(modifiers.logo())))
      }
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => {
        self.game_exited.store(true, Ordering::Release);
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
        self.ecs.get_events_mut::<InputEvent>().send(InputEvent::KeyBoard(MyKeyboardInput::Key(input.virtual_keycode)));
      }
      Event::WindowEvent {
        event: WindowEvent::CursorMoved { position, .. },
        ..
      } => {
        self.ecs.get_events_mut::<InputEvent>().send(InputEvent::Mouse(MyMouseInput {
          position
        }));
      }
      _ => (),
    }
  }

  fn status_string(&self) -> String {
    let camera_status = self.camera.to_string(&self.ecs.world);
    let avg = self.frame_times_avg.count();
    let all_avg = self.frame_times_avg.all_count();
    format!("camera {}\nmyworld {}\navgftw {:.2} navgft {:.2} ", camera_status, self.myworld, avg, all_avg)
  }
}

