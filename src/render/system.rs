use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass};
use vulkano::image::{AttachmentImage, ImmutableImage, SwapchainImage};
use vulkano::image::view::{ImageView, ImageViewType};
use vulkano::pipeline::depth_stencil::{Compare, DepthStencil};
use vulkano::pipeline::vertex::TwoBuffersDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::GpuFuture;
use profiling;

use cgmath::Point3;

use winit::window::Window;

use std::convert::TryInto;
use std::iter;
use std::sync::Arc;

use crate::render::scene::MergedScene;
use crate::render::scene::Scene;
use crate::render::skybox::SkyboxCubemap;
use crate::render::textures::Textures;
use crate::shaders;
use crate::utils::{Normal, Vertex};
use crate::Graph;

pub struct System {
  text_texture: Arc<ImmutableImage>,
  text_sampler: Arc<Sampler>,
  skybox_cubemap: SkyboxCubemap,
  pub pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
  pub pipeline_skybox: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
  pub framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
  uniform_buffer: CpuBufferPool<shaders::main::vs::ty::Data>,
  uniform_skybox_buffer: CpuBufferPool<shaders::skybox::vs::ty::Data>,
  environment_buffer: CpuBufferPool<shaders::main::fs::ty::Environment>,
  point_lights_buffer: CpuBufferPool<shaders::main::fs::ty::PointLights>,
  directional_lights_buffer: CpuBufferPool<shaders::main::fs::ty::DirectionalLights>,
  spot_lights_buffer: CpuBufferPool<shaders::main::fs::ty::SpotLights>,
  color_buffer: Arc<AttachmentImage>,
  depth_buffer: Arc<AttachmentImage>,
}

impl System {
  pub fn new(graph: &Graph, textures: Textures) -> (Self, Box<dyn GpuFuture>) {
    let (text_texture, text_future) = textures.draw(&graph.queue);
    let text_sampler = Sampler::new(
      graph.device.clone(),
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
    let (skybox_cubemap, tex_future) = SkyboxCubemap::new(&graph.queue);

    let (pipeline, pipeline_skybox, framebuffers, color_buffer, depth_buffer) =
      window_size_dependent_setup(
        graph.device.clone(),
        &graph.vs,
        &graph.fs,
        &graph.skybox_vs,
        &graph.skybox_fs,
        &graph.images,
        graph.render_pass.clone(),
      );

    let uniform_buffer =
      CpuBufferPool::<shaders::main::vs::ty::Data>::new(graph.device.clone(), BufferUsage::all());
    let uniform_skybox_buffer =
      CpuBufferPool::<shaders::skybox::vs::ty::Data>::new(graph.device.clone(), BufferUsage::all());

    let environment_buffer = CpuBufferPool::<shaders::main::fs::ty::Environment>::new(
      graph.device.clone(),
      BufferUsage::all(),
    );
    let point_lights_buffer = CpuBufferPool::<shaders::main::fs::ty::PointLights>::new(
      graph.device.clone(),
      BufferUsage::all(),
    );
    let directional_lights_buffer = CpuBufferPool::<shaders::main::fs::ty::DirectionalLights>::new(
      graph.device.clone(),
      BufferUsage::all(),
    );
    let spot_lights_buffer = CpuBufferPool::<shaders::main::fs::ty::SpotLights>::new(
      graph.device.clone(),
      BufferUsage::all(),
    );

    (
      System {
        text_texture,
        text_sampler,
        skybox_cubemap,
        pipeline,
        pipeline_skybox,
        framebuffers,
        uniform_buffer,
        uniform_skybox_buffer,
        environment_buffer,
        point_lights_buffer,
        directional_lights_buffer,
        spot_lights_buffer,
        color_buffer,
        depth_buffer,
      },
      text_future.join(tex_future).boxed(),
    )
  }

  #[profiling::function]
  pub fn main_set(
    &self,
    proj: shaders::main::vs::ty::Data,
    scenes: Vec<&Scene>,
    camera_position: Point3<f32>,
  ) -> Arc<dyn DescriptorSet + Sync + Send> {
    let uniform_buffer_subbuffer = {
      let uniform_data = proj;
      self.uniform_buffer.next(uniform_data).unwrap()
    };

    let mut all_scene = MergedScene::default();
    for scene in scenes {
      all_scene
        .point_lights
        .extend(scene.point_lights.iter().map(|arc| arc.as_ref()));
      all_scene
        .directional_lights
        .extend(scene.directional_lights.iter().map(|arc| arc.as_ref()));
      all_scene
        .spot_lights
        .extend(scene.spot_lights.iter().map(|arc| arc.as_ref()));
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

    let layout = self.pipeline.descriptor_set_layout(0).unwrap();
    let text_image = ImageView::new(self.text_texture.clone()).unwrap();

    Arc::new(
      PersistentDescriptorSet::start(layout.clone())
        .add_buffer(uniform_buffer_subbuffer)
        .unwrap()
        .add_sampled_image(text_image, self.text_sampler.clone())
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
    )
  }

  #[profiling::function]
  pub fn skybox_set(
    &self,
    proj: shaders::skybox::vs::ty::Data,
  ) -> Arc<dyn DescriptorSet + Sync + Send> {
    let layout = self.pipeline_skybox.descriptor_set_layout(0).unwrap();
    let uniform_buffer_subbuffer = {
      let uniform_data = proj;
      self.uniform_skybox_buffer.next(uniform_data).unwrap()
    };

    let color_buffer_view = ImageView::new(self.color_buffer.clone()).unwrap();
    let depth_buffer_view = ImageView::new(self.depth_buffer.clone()).unwrap();
    let skybox_texture_view = ImageView::start(self.skybox_cubemap.texture.clone())
      .with_type(ImageViewType::Cubemap)
      .build()
      .unwrap();
    Arc::new(
      PersistentDescriptorSet::start(layout.clone())
        .add_buffer(uniform_buffer_subbuffer)
        .unwrap()
        .add_image(color_buffer_view)
        .unwrap()
        .add_image(depth_buffer_view)
        .unwrap()
        .add_sampled_image(
          skybox_texture_view,
          self.skybox_cubemap.sampler.clone(),
        )
        .unwrap()
        .build()
        .unwrap(),
    )
  }

  #[profiling::function]
  pub fn recreate_swapchain(&mut self, graph: &Graph) {
    let (pipeline, pipeline_skybox, framebuffers, color_buffer, depth_buffer) =
      window_size_dependent_setup(
        graph.device.clone(),
        &graph.vs,
        &graph.fs,
        &graph.skybox_vs,
        &graph.skybox_fs,
        &graph.images,
        graph.render_pass.clone(),
      );

    self.pipeline = pipeline;
    self.pipeline_skybox = pipeline_skybox;
    self.framebuffers = framebuffers;

    self.color_buffer = color_buffer;
    self.depth_buffer = depth_buffer;
  }
}

#[profiling::function]
fn window_size_dependent_setup(
  device: Arc<Device>,
  vs: &shaders::main::vs::Shader,
  fs: &shaders::main::fs::Shader,
  skybox_vs: &shaders::skybox::vs::Shader,
  skybox_fs: &shaders::skybox::fs::Shader,
  images: &[Arc<SwapchainImage<Window>>],
  render_pass: Arc<RenderPass>,
) -> (
  Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
  Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
  Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
  Arc<AttachmentImage>,
  Arc<AttachmentImage>,
) {
  let dimensions = images[0].dimensions();

  let depth_buffer =
    AttachmentImage::input_attachment(device.clone(), dimensions, Format::D16Unorm).unwrap();

  let color_buffer =
    AttachmentImage::input_attachment(device.clone(), dimensions, Format::B8G8R8A8Unorm).unwrap();

  let depth_buffer2 =
    AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

  let framebuffers = images
    .iter()
    .map(|image| {
      let view = ImageView::new(image.clone()).unwrap();
      let depth_view = ImageView::new(depth_buffer.clone()).unwrap();
      let depth2_view = ImageView::new(depth_buffer2.clone()).unwrap();
      let color_view = ImageView::new(color_buffer.clone()).unwrap();
      Arc::new(
        Framebuffer::start(render_pass.clone())
          .add(view)
          .unwrap()
          .add(depth_view)
          .unwrap()
          .add(color_view)
          .unwrap()
          .add(depth2_view)
          .unwrap()
          .build()
          .unwrap(),
      ) as Arc<dyn FramebufferAbstract + Send + Sync>
    })
    .collect::<Vec<_>>();

  // In the triangle example we use a dynamic viewport, as its a simple example.
  // However in the teapot example, we recreate the pipelines with a hardcoded viewport instead.
  // This allows the driver to optimize things, at the cost of slower window resizes.
  // https://computergraphics.stackexchange.com/questions/5742/vulkan-best-way-of-updating-pipeline-viewport
  let pipeline = Arc::new(
    GraphicsPipeline::start()
      .vertex_input(TwoBuffersDefinition::<Vertex, Normal>::new())
      .vertex_shader(vs.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .viewports(iter::once(Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
      }))
      .fragment_shader(fs.main_entry_point(), ())
      .depth_stencil_simple_depth()
      .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
      .build(device.clone())
      .unwrap(),
  );

  let mut depth_stencil = DepthStencil::simple_depth_test();
  depth_stencil.depth_compare = Compare::LessOrEqual;

  let pipeline_skybox = Arc::new(
    GraphicsPipeline::start()
      .vertex_input(TwoBuffersDefinition::<Vertex, Normal>::new())
      .vertex_shader(skybox_vs.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .viewports(iter::once(Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
      }))
      .fragment_shader(skybox_fs.main_entry_point(), ())
      .depth_stencil(depth_stencil)
      .render_pass(Subpass::from(render_pass.clone(), 1).unwrap())
      .build(device)
      .unwrap(),
  );

  (
    pipeline,
    pipeline_skybox,
    framebuffers,
    color_buffer,
    depth_buffer,
  )
}
