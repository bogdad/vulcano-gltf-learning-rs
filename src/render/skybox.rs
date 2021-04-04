use image::ImageFormat;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImmutableImage, MipmapsCount};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::GpuFuture;

use std::sync::Arc;

pub struct SkyboxCubemap {
  pub texture: Arc<ImmutableImage<Format>>,
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
    for image in cubemap_images.iter() {
      let mut image0 = image.clone().into_raw().clone();
      image_data.append(&mut image0);
    }

    let (texture, future) = {
      ImmutableImage::from_iter(
        image_data.iter().cloned(),
        Dimensions::Cubemap { size: 1024 },
        MipmapsCount::Specific(6),
        Format::R8G8B8A8Srgb,
        queue.clone(),
      )
      .unwrap()
    };

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
