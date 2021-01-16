use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;

use vulkano::command_buffer::{AutoCommandBufferBuilder, SubpassContents};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use cgmath::prelude::*;
use cgmath::{Matrix3, Matrix4, Point3, Rad, Vector3};

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::vs;
use crate::Graph;
use crate::Model;

use crate::terrain_generation;

#[derive(Debug)]
struct Camera {
  front: Vector3<f32>,
  up: Vector3<f32>,
  pos: Point3<f32>,
}
impl Camera {
  pub fn react(self: &mut Camera, input: &KeyboardInput) -> bool {
    if let KeyboardInput {
      virtual_keycode: Some(key_code),
      ..
    } = input
    {
      let camera_speed = 0.25;
      let zz = self.front.cross(self.up).normalize();
      match key_code {
        VirtualKeyCode::A => {
          self.pos -= zz * camera_speed;
          return true;
        }
        VirtualKeyCode::D => {
          self.pos += zz * camera_speed;
          return true;
        }
        VirtualKeyCode::W => {
          self.pos += camera_speed * self.front;
          return true;
        }
        VirtualKeyCode::S => {
          self.pos -= camera_speed * self.front;
          return true;
        }
        _ => {
          return false;
        }
      };
    }
    return false;
  }
}

struct World {}
impl World {
  pub fn camera_entered(&mut self, pos: &Point3<f32>) {
    // entering
    if pos.x.rem_euclid(2.0) < f32::EPSILON && pos.z.rem_euclid(2.0) < f32::EPSILON {
      println!(" entering x,z {:?} {:?}", pos.x, pos.z);
    }
  }
}

pub struct Game {
  graph: Graph,
  camera: Camera,
  world: World,
  recreate_swapchain: bool,
  models: Vec<Model>,
  uniform_buffer: CpuBufferPool<vs::ty::Data>,
  previous_frame_end: Option<Box<dyn GpuFuture>>,
  rotation_start: Instant,
}

impl Game {
  pub fn new(graph: Graph) -> Game {
    // gltf:
    // "and the default camera sits on the
    // -Z side looking toward the origin with +Y up"
    //                               x     y    z
    let camera = Camera {
      pos: Point3::new(0.0, -0.2, -1.0),
      front: Vector3::new(0.0, 0.0, 1.0),
      up: Vector3::new(0.0, 1.0, 0.0),
    };

    let world = World {};

    let recreate_swapchain = false;
    let previous_frame_end = Some(sync::now(graph.device.clone()).boxed());

    let rotation_start = Instant::now();

    let models = vec![
      //Model::from_gltf(Path::new("models/creature.glb"), &device),
      //Model::from_gltf(Path::new("models/creature2.glb"), &device),
      //Model::from_gltf(Path::new("models/creature3.glb"), &device),
      //Model::from_gltf(Path::new("models/landscape.glb"), &graph.device),
      Model::from_gltf(Path::new("models/dog.glb"), &graph.device),
      //Model::from_gltf(Path::new("models/box.glb"), &device),
      //Model::from_gltf(Path::new("models/center.glb"), &device),
      terrain_generation::execute(128, 12).get_buffers(&graph.device),
    ];

    let uniform_buffer =
      CpuBufferPool::<vs::ty::Data>::new(graph.device.clone(), BufferUsage::all());

    Game {
      graph,
      camera,
      world,
      recreate_swapchain,
      models,
      uniform_buffer,
      previous_frame_end,
      rotation_start,
    }
  }

  fn draw(&mut self) {
    self.previous_frame_end.as_mut().unwrap().cleanup_finished();
    if self.recreate_swapchain {
      self.graph.recreate_swapchain();
      self.recreate_swapchain = false;
    }
    let uniform_buffer_subbuffer = {
      let _elapsed = self.rotation_start.elapsed();
      let rotation = 0;
      //elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
      let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

      // note: this teapot was meant for OpenGL where the origin is at the lower left
      //       instead the origin is at the upper left in Vulkan, so we reverse the Y axis
      let aspect_ratio = self.graph.dimensions[0] as f32 / self.graph.dimensions[1] as f32;
      let mut proj =
        cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);

      // flipping the "horizontal" projection bit
      proj[0][0] = -proj[0][0];

      let target = self.camera.pos.to_vec() + self.camera.front;

      let view = Matrix4::look_at(self.camera.pos, Point3::from_vec(target), self.camera.up);
      let scale = Matrix4::from_scale(0.01);
      /*
         mat4 worldview = uniforms.view * uniforms.world;
         v_normal = transpose(inverse(mat3(worldview))) * normal;
         gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
      */
      let uniform_data = vs::ty::Data {
        //world: Matrix4::from(eye).into(),
        world: Matrix4::from(rotation).into(),
        //world: <Matrix4<f32> as One>::one().into(),
        view: (view * scale).into(),
        proj: proj.into(),
      };

      self.uniform_buffer.next(uniform_data).unwrap()
    };
    let layout = self.graph.pipeline.descriptor_set_layout(0).unwrap();
    let set = Arc::new(
      PersistentDescriptorSet::start(layout.clone())
        .add_buffer(uniform_buffer_subbuffer)
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

    builder.end_render_pass().unwrap();
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
        let camera_moved = self.camera.react(&input);
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
}
