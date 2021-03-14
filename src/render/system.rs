use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::{AttachmentImage, ImageUsage, ImmutableImage, SwapchainImage};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::GpuFuture;

use cgmath::{EuclideanSpace, InnerSpace, Matrix3, Matrix4, Point3, Rad, Vector3};

use std::convert::TryInto;
use std::sync::Arc;

use crate::render::model::ModelScene;
use crate::render::scene::MergedScene;
use crate::render::skybox::SkyboxCubemap;
use crate::render::textures::Textures;
use crate::shaders;

pub struct System {
  text_texture: Arc<ImmutableImage<Format>>,
  text_sampler: Arc<Sampler>,
  skybox_cubemap: SkyboxCubemap,
  uniform_buffer: CpuBufferPool<shaders::main::vs::ty::Data>,
  environment_buffer: CpuBufferPool<shaders::main::fs::ty::Environment>,
  point_lights_buffer: CpuBufferPool<shaders::main::fs::ty::PointLights>,
  directional_lights_buffer: CpuBufferPool<shaders::main::fs::ty::DirectionalLights>,
  spot_lights_buffer: CpuBufferPool<shaders::main::fs::ty::SpotLights>,
}

impl System {
  pub fn new(
    device: &Arc<Device>,
    queue: &Arc<Queue>,
    dimensions: [u32; 2],
    textures: Textures,
  ) -> (Self, Box<dyn GpuFuture>) {
    let (text_texture, text_future) = textures.draw(queue);
    let text_sampler = Sampler::new(
      device.clone(),
      Filter::Linear,
      Filter::Linear,
      MipmapMode::Nearest,
      SamplerAddressMode::ClampToEdge,
      SamplerAddressMode::ClampToEdge,
      SamplerAddressMode::ClampToEdge,
      0.0,
      1.0,
      0.0,
      1.0,
    )
    .unwrap();
    let skybox_cubemap = SkyboxCubemap::new(queue);
    let uniform_buffer =
      CpuBufferPool::<shaders::main::vs::ty::Data>::new(device.clone(), BufferUsage::all());
    let environment_buffer =
      CpuBufferPool::<shaders::main::fs::ty::Environment>::new(device.clone(), BufferUsage::all());
    let point_lights_buffer =
      CpuBufferPool::<shaders::main::fs::ty::PointLights>::new(device.clone(), BufferUsage::all());
    let directional_lights_buffer = CpuBufferPool::<shaders::main::fs::ty::DirectionalLights>::new(
      device.clone(),
      BufferUsage::all(),
    );
    let spot_lights_buffer =
      CpuBufferPool::<shaders::main::fs::ty::SpotLights>::new(device.clone(), BufferUsage::all());

    let color_buffer =
      AttachmentImage::input_attachment(device.clone(), dimensions, Format::B8G8R8A8Unorm).unwrap();

    let depth_buffer =
      AttachmentImage::input_attachment(device.clone(), dimensions, Format::D16Unorm).unwrap();

    (
      System {
        text_texture,
        text_sampler,
        skybox_cubemap,
        uniform_buffer,
        environment_buffer,
        point_lights_buffer,
        directional_lights_buffer,
        spot_lights_buffer,
      },
      text_future,
    )
  }

  pub fn main_set(
    &self,
    proj: shaders::main::vs::ty::Data,
    models: &Vec<ModelScene>,
    camera_position: Point3<f32>,
    pipeline: &Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
  ) -> Arc<dyn DescriptorSet + Sync + Send> {
    let uniform_buffer_subbuffer = {
      let uniform_data = proj;
      self.uniform_buffer.next(uniform_data).unwrap()
    };

    let mut all_scene = MergedScene::default();
    for model in models {
      all_scene
        .point_lights
        .extend(model.1.point_lights.iter().map(|arc| arc.as_ref()));
      all_scene
        .directional_lights
        .extend(model.1.directional_lights.iter().map(|arc| arc.as_ref()));
      all_scene
        .spot_lights
        .extend(model.1.spot_lights.iter().map(|arc| arc.as_ref()));
    }

    let environment_buffer_subbuffer = {
      let environment = shaders::main::fs::ty::Environment {
        ambient_color: [0.0, 0.0, 0.0],
        camera_position: camera_position.into(),
        point_light_count: all_scene.point_lights.len() as i32,
        directional_light_count: all_scene.directional_lights.len() as i32,
        spot_light_count: all_scene.spot_lights.len() as i32,
        sun_color: [0.6, 0.6, 0.65],
        sun_direction: [-0.577, -0.577, -0.577],
        ..Default::default()
      };
      self.environment_buffer.next(environment).unwrap()
    };
    all_scene.point_lights.reserve_exact(128);
    for _i in all_scene.point_lights.len()..128 {
      all_scene.point_lights.push(Default::default());
    }
    all_scene.spot_lights.reserve_exact(128);
    for _i in all_scene.spot_lights.len()..128 {
      all_scene.spot_lights.push(Default::default());
    }
    all_scene.directional_lights.reserve_exact(16);
    for _i in all_scene.directional_lights.len()..16 {
      all_scene.directional_lights.push(Default::default());
    }

    let point_lights_buffer_subbuffer = {
      let point_lights = shaders::main::fs::ty::PointLights {
        plight: all_scene.point_lights.as_slice().try_into().unwrap(),
      };
      self.point_lights_buffer.next(point_lights).unwrap()
    };

    let directional_lights_buffer_subbuffer = {
      let directional_lights = shaders::main::fs::ty::DirectionalLights {
        dlight: all_scene.directional_lights.as_slice().try_into().unwrap(),
      };
      self
        .directional_lights_buffer
        .next(directional_lights)
        .unwrap()
    };

    let spot_lights_buffer_subbuffer = {
      let spot_lights = shaders::main::fs::ty::SpotLights {
        slight: all_scene.spot_lights.as_slice().try_into().unwrap(),
      };
      self.spot_lights_buffer.next(spot_lights).unwrap()
    };

    let layout = pipeline.descriptor_set_layout(0).unwrap();

    let set = Arc::new(
      PersistentDescriptorSet::start(layout.clone())
        .add_buffer(uniform_buffer_subbuffer)
        .unwrap()
        .add_sampled_image(self.text_texture.clone(), self.text_sampler.clone())
        .unwrap()
        .add_buffer(environment_buffer_subbuffer)
        .unwrap()
        .add_buffer(point_lights_buffer_subbuffer)
        .unwrap()
        .add_buffer(directional_lights_buffer_subbuffer)
        .unwrap()
        .add_buffer(spot_lights_buffer_subbuffer)
        .unwrap()
        .build()
        .unwrap(),
    );
    set
  }

  pub fn skybox_set(
    &self,
    proj: shaders::main::vs::ty::Data,
    pipeline_skybox: &Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    color_buffer: &Arc<AttachmentImage>,
    depth_buffer: &Arc<AttachmentImage>,
  ) -> Arc<dyn DescriptorSet + Sync + Send> {
    let layout = pipeline_skybox.descriptor_set_layout(0).unwrap();
    let uniform_buffer_subbuffer = {
      let uniform_data = proj;
      self.uniform_buffer.next(uniform_data).unwrap()
    };
    let set = Arc::new(
      PersistentDescriptorSet::start(layout.clone())
        .add_buffer(uniform_buffer_subbuffer)
        .unwrap()
        .add_image(color_buffer.clone())
        .unwrap()
        .add_image(depth_buffer.clone())
        .unwrap()
        .build()
        .unwrap(),
    );
    set
  }
}
