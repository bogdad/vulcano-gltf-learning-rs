use vulkano::image::{Dimensions, ImmutableImage, MipmapsCount, SwapchainImage};
use vulkano::format::Format;
use vulkano::device::Queue;
use vulkano::sync::GpuFuture;
use image::{Pixel, RgbImage};

use std::sync::Arc;

use crate::things::texts::Texts;

pub struct Textures {
    texture: RgbImage,
}


impl Textures {
    pub fn new(texts: &Texts) -> Textures {
      let texture = texts.texture();
      Textures {
        texture
      }
    }

    pub fn draw(&self, queue: &Arc<Queue>) -> (Arc<ImmutableImage<Format>>, Box<dyn GpuFuture>) {
      let (texture, future) = {

        let dimensions = Dimensions::Dim2d {
            width: self.texture.dimensions().0,
            height: self.texture.dimensions().1,
        };

        ImmutableImage::from_iter(
            self.texture.pixels().map(|p| (
              p.to_rgba().channels4().0,
              p.to_rgba().channels4().1,
              p.to_rgba().channels4().2,
              p.to_rgba().channels4().3,
              )),
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
