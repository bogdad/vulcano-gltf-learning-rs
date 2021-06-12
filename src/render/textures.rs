use image::{Pixel, RgbImage};
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount};
use vulkano::sync::GpuFuture;

use std::sync::Arc;

use crate::things::texts::Texts;

pub struct Textures {
  texture: RgbImage,
}

impl Textures {
  pub fn new(texts: &Texts) -> Textures {
    let texture = texts.texture();
    Textures { texture }
  }

  pub fn draw(&self, queue: &Arc<Queue>) -> (Arc<ImmutableImage>, Box<dyn GpuFuture>) {
    let (texture, future) = {
      let dimensions = ImageDimensions::Dim2d {
        width: self.texture.dimensions().0,
        height: self.texture.dimensions().1,
        array_layers: 1,
      };
      println!("texture dimensions {:?}", dimensions);
      ImmutableImage::from_iter(
        self.texture.pixels().map(|p| {
          if p.to_rgba().channels()[0] < 50 {
            (0, 0, 0, 255)
          } else {
            (
              p.to_rgba().channels4().0,
              p.to_rgba().channels4().1,
              p.to_rgba().channels4().2,
              p.to_rgba().channels4().3,
            )
          }
        }),
        dimensions,
        MipmapsCount::One,
        Format::R8G8B8A8Srgb,
        queue.clone(),
      )
      .unwrap()
    };
    (texture, Box::new(future))
  }
}
