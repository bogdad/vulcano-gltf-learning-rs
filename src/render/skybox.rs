use image::ImageFormat;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount, ImageUsage, ImageCreateFlags, ImageLayout};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::GpuFuture;

use std::sync::Arc;

pub struct SkyboxCubemap {
  pub texture: Arc<ImmutableImage>,
  pub sampler: Arc<Sampler>,
}

impl SkyboxCubemap {
  pub fn new(queue: &Arc<Queue>) -> (Self, Box<dyn GpuFuture>) {
    let img_negx = image::load_from_memory_with_format(
      include_bytes!("../../assets/interstellar_skybox/xneg.png"),
      ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let img_posx = image::load_from_memory_with_format(
      include_bytes!("../../assets/interstellar_skybox/xpos.png"),
      ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let img_posy = image::load_from_memory_with_format(
      include_bytes!("../../assets/interstellar_skybox/ypos.png"),
      ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let img_negy = image::load_from_memory_with_format(
      include_bytes!("../../assets/interstellar_skybox/yneg.png"),
      ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let img_negz = image::load_from_memory_with_format(
      include_bytes!("../../assets/interstellar_skybox/zneg.png"),
      ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let img_posz = image::load_from_memory_with_format(
      include_bytes!("../../assets/interstellar_skybox/zpos.png"),
      ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();

    let cubemap_images = [img_posx, img_negx, img_posy, img_negy, img_posz, img_negz];
    let mut image_data: Vec<u8> = Vec::new();
    let mut height = 0;
    let mut width = 0;
    for image in cubemap_images.iter() {
      let (w, h) = image.dimensions();
      height = h;
      width = width + w;
      let mut image0 = image.clone().into_raw().clone();
      image_data.append(&mut image0);
    }

    let source = CpuAccessibleBuffer::from_iter(
      queue.device().clone(),
      BufferUsage::transfer_source(),
      false,
      image_data.iter().cloned(),
    )
    .unwrap();

    let dimensions = ImageDimensions::Dim2d {
      width: 1,
      height: 1,
      array_layers: 6,
  };

    let (texture, _) = ImmutableImage::uninitialized(
      queue.device().clone(),
      dimensions,
      Format::R8G8B8A8Srgb,
      MipmapsCount::One,
      ImageUsage {
          transfer_destination: true,
          sampled: true,
          ..ImageUsage::none()
      },
      ImageCreateFlags {
          cube_compatible: true,
          ..ImageCreateFlags::none()
      },
      ImageLayout::ShaderReadOnlyOptimal,
      queue.device().active_queue_families(),
    )
    .unwrap();

    let sampler = Sampler::new(
      queue.device().clone(),
      Filter::Linear,
      Filter::Linear,
      MipmapMode::Nearest,
      SamplerAddressMode::ClampToEdge,
      SamplerAddressMode::ClampToEdge,
      SamplerAddressMode::ClampToEdge,
      0.0,
      1.0,
      0.0,
      0.0,
    )
    .unwrap();
    (SkyboxCubemap { texture, sampler }, future.boxed())
  }
}
