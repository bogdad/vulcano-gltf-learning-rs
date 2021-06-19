use cgmath::{EuclideanSpace, Matrix3, Matrix4, Point3, Rad, Vector3};


use crate::shaders;
use crate::Graph;

use bevy_ecs::world::World;
use bevy_ecs::entity::Entity;
use crate::components::{CameraBundle, Position, CameraId};
use crate::ecs::Ecs;

#[derive(Debug)]
pub struct Camera {
  camera_entity: Entity,
}

impl Camera {

  pub fn new(ecs: &mut Ecs) -> Self {
    let camera_entity = ecs.world.spawn().insert_bundle(CameraBundle {
      position: Position { point3: Point3::new(0.0, -1.0, -1.0) },
      camera: CameraId {
        front: Vector3::new(0.0, 0.0, 1.0),
        up: Vector3::new(0.0, 1.0, 0.0),
        speed: 0.3,
        last_x: None,
        last_y: None,
        yaw: 0.0,
        pitch: 0.0,
      },
      ..Default::default()
    }).id();
    Camera {
      camera_entity: camera_entity,
    }
  }

  pub fn get_pos(&self, world: &World) -> Point3<f32> {
    world.get_entity(self.camera_entity).unwrap().get::<Position>().unwrap().point3
  }

  pub fn proj(&self, graph: &Graph, world: &World) -> shaders::main::vs::ty::Data {

    let pos = world.get_entity(self.camera_entity).unwrap().get::<Position>().unwrap().point3;
    let camera_id = world.get_entity(self.camera_entity).unwrap().get::<CameraId>().unwrap();

    //let _elapsed = self.rotation_start.elapsed();
    let rotation = 0;
    //elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
    let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

    // note: this teapot was meant for OpenGL where the origin is at the lower left
    //       instead the origin is at the upper left in, Vulkan, so we reverse the Y axis
    let aspect_ratio = graph.dimensions[0] as f32 / graph.dimensions[1] as f32;
    let mut proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.1, 100.0);

    // flipping the "horizontal" projection bit
    proj[0][0] = -proj[0][0];

    let target = pos.to_vec() + camera_id.front;

    let view = Matrix4::look_at_rh(pos, Point3::from_vec(target), camera_id.up);
    let scale = Matrix4::from_scale(0.99);
    /*
       mat4 worldview = uniforms.view * uniforms.world;
       v_normal = transpose(inverse(mat3(worldview))) * normal;
       gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
    */
    shaders::main::vs::ty::Data {
      //world: Matrix4::from(eye).into(),
      world: Matrix4::from(rotation).into(),
      //world: <Matrix4<f32> as One>::one().into(),
      view: (view * scale).into(),
      proj: proj.into(),
      camera_position: pos.into(),
    }
  }

  pub fn proj_skybox(&self, graph: &Graph, world: &World) -> shaders::skybox::vs::ty::Data {
    let pos = world.get_entity(self.camera_entity).unwrap().get::<Position>().unwrap().point3;
    let camera_id = world.get_entity(self.camera_entity).unwrap().get::<CameraId>().unwrap();

    //let _elapsed = self.rotation_start.elapsed();
    let rotation = 0;
    //elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
    let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

    // note: this teapot was meant for OpenGL where the origin is at the lower left
    //       instead the origin is at the upper left in, Vulkan, so we reverse the Y axis
    let aspect_ratio = graph.dimensions[0] as f32 / graph.dimensions[1] as f32;
    let mut proj = cgmath::perspective(
      Rad(std::f32::consts::FRAC_PI_2),
      aspect_ratio,
      0.001,
      100000.0,
    );

    // flipping the "horizontal" projection bit
    proj[0][0] = -proj[0][0];

    let target = pos.to_vec() + camera_id.front;

    let view = Matrix4::look_at_rh(pos, Point3::from_vec(target), camera_id.up);
    let scale = Matrix4::from_scale(0.99);
    /*
       mat4 worldview = uniforms.view * uniforms.world;
       v_normal = transpose(inverse(mat3(worldview))) * normal;
       gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
    */
    shaders::skybox::vs::ty::Data {
      //world: Matrix4::from(eye).into(),
      world: Matrix4::from(rotation).into(),
      //world: <Matrix4<f32> as One>::one().into(),
      view: (view * scale).into(),
      proj: proj.into(),
      camera_position: pos.into(),
    }
  }

  pub fn to_string(&self, world: &World) -> String {
    let pos = world.get_entity(self.camera_entity).unwrap().get::<Position>().unwrap().point3;
    let camera_id = world.get_entity(self.camera_entity).unwrap().get::<CameraId>().unwrap();
    format!("camera {:?}", pos)
  }
}