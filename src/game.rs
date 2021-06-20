use crate::input::GameEvent;
use crate::input::GameWantsExitEvent;
use crate::input::InputEvent;
use bevy_ecs::event::ManualEventReader;
use cgmath::Point3;
use profiling;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_text::DrawTextTrait;

use std::boxed::Box;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::vec::Vec;

use crate::camera::Camera;
use crate::ecs::Ecs;
use crate::executor::Executor;
use crate::myworld::MyWorld;
use crate::render::System;
use crate::render::Textures;
use crate::sign_post::SignPost;
use crate::sounds::Sounds;
use crate::things::CountingWindowAvg;
use crate::things::Lap;
use crate::things::Texts;
use crate::things::{PrimitiveCube, PrimitiveTriangle};
use crate::Graph;
use crate::Model;
use crate::Settings;

pub struct Game {
  ecs: Ecs,
  events_reader: Option<ManualEventReader<GameEvent>>,

  system: System,
  camera: Camera,
  settings: Settings,
  graph: Graph,
  sounds: Sounds,

  myworld: MyWorld,
  recreate_swapchain: bool,
  previous_frame_end: Option<Box<dyn GpuFuture>>,
  models: Vec<Model>,
  i_frame: u64,
  last_frame_took: u32,

  pub game_exited: Arc<AtomicBool>,
  frame_times_avg: CountingWindowAvg,
  recv: Receiver<InputEvent>,
}

impl Game {
  pub fn new(
    settings: Settings,
    executor: Executor,
    graph: Graph,
    game_exited: Arc<AtomicBool>,
    recv: Receiver<InputEvent>,
  ) -> Game {
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

    let last_frame_took = 0;

    let frame_times_avg = CountingWindowAvg::new(30);

    let sounds = Sounds::new();

    Game {
      ecs,
      events_reader: None,
      settings,
      graph,
      sounds,
      camera,
      myworld,
      recreate_swapchain,
      previous_frame_end,
      models,
      system,
      i_frame: 0,
      last_frame_took,
      game_exited,
      frame_times_avg,
      recv,
    }
  }

  #[profiling::function]
  pub fn game_loop(&mut self) {
    let events = self.ecs.get_events::<GameEvent>();
    let mut reader = events.get_reader();
    self.init();
    while !self.game_exited.load(Ordering::Acquire) {
      self.wait_for_frame();
      {
        self.accept_events(&mut reader);
        self.tick();
        self.draw();
        profiling::finish_frame!();
      }
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
      //if self.i_frame % 2 == 0 {
      self.previous_frame_end.as_mut().unwrap().cleanup_finished();
      //}
    }
    if self.recreate_swapchain {
      profiling::scope!("recreate_swap_chain");
      self.previous_frame_end.as_mut().unwrap().cleanup_finished();
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
      self
        .system
        .skybox_set(self.camera.proj_skybox(&self.graph, &self.ecs.world))
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
    self.last_frame_took = last_frame;
    self.frame_times_avg.add(last_frame);
  }

  #[profiling::function]
  pub fn tick(&mut self) {
    self.myworld.tick(&self.ecs);
    self.ecs.tick();
  }

  #[profiling::function]
  pub fn accept_events(&mut self, game_event_reader: &mut ManualEventReader<GameEvent>) {
    loop {
      let events = self.ecs.get_events::<GameEvent>();
      for event in game_event_reader.iter(events) {
        match event {
          GameEvent::Game(GameWantsExitEvent {}) => {
            self.game_exited.store(true, Ordering::Release);
          }
          _ => {}
        }
      }
      let next = self.recv.try_recv();
      if next.is_err() {
        break;
      }
      let event = next.unwrap();
      match event {
        InputEvent::RecreateSwapchain => self.recreate_swapchain = true,
        _ => self.ecs.get_events_mut().send(event),
      }
    }
  }

  #[profiling::function]
  pub fn wait_for_frame(&self) {
    let last_frame_took = self.last_frame_took;
    // 1000 ms / 30 fps = 33 ms
    let last_frame_took_duration = Duration::from_millis(last_frame_took as u64);
    let interval = std::time::Duration::from_millis(33);
    if interval > last_frame_took_duration {
      profiling::scope!("sleeping");
      let sleep = interval - last_frame_took_duration;
      let now = Instant::now();
      mysleep_until(now, now + sleep);
    } else {
      println!("last frame was {}", last_frame_took_duration.as_millis());
    }
  }

  fn status_string(&self) -> String {
    let camera_status = self.camera.to_string(&self.ecs.world);
    let avg = self.frame_times_avg.count();
    let all_avg = self.frame_times_avg.all_count();
    format!(
      "camera {}\nmyworld {}\navgftw {:.2} navgft {:.2} ",
      camera_status, self.myworld, avg, all_avg
    )
  }
}

#[profiling::function]
pub fn mysleep_until(now: Instant, t: Instant) {
  let mut cur = now;
  while cur < t {
    if t - cur > Duration::from_millis(15) {
      std::thread::sleep(Duration::from_millis(10));
    } else {
      std::thread::sleep(Duration::from_millis(0));
    }
    cur = Instant::now();
  }
}
