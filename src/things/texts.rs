extern crate ab_glyph;

use ab_glyph::{FontRef, Font, Glyph, point};
use image::{RgbImage, Rgb, GenericImage};

use std::collections::HashMap;

struct Image {
  image: RgbImage,
}

impl Image {
  pub fn from_text(text: &str) -> Self {
    let font = FontRef::try_from_slice(include_bytes!("../../fonts/kleymissky.otf")).unwrap();
    // Get a glyph for 'q' with a scale & position.
    let mut x = 0.0;
    let mut y = 0.0;
    for chr in text.chars() {
      let glyph: Glyph = font.glyph_id(chr).with_scale_and_position(24.0, point(0.0, 0.0));
      let q = font.outline_glyph(glyph).unwrap();
      x += q.px_bounds().max.x;
      if y <= q.px_bounds().max.y {
        y = q.px_bounds().max.y
      }
    }
    let w = x as u32;
    let h = y as u32;
    let mut image = RgbImage::new(w, h);
    x = 0.0;
    for chr in text.chars() {
      let glyph: Glyph = font.glyph_id(chr).with_scale_and_position(24.0, point(x, 0.0));
      let q = font.outline_glyph(glyph).unwrap();
      x = q.px_bounds().max.x;
      q.draw(|x, y, c| { image.put_pixel(x, y, Rgb([(255.0 * c) as u8, 0, 0])) });
    }

    Image {
      image
    }
  }
}

struct ImageInfo {
  rect: (u32, u32),
}

pub struct Texts {
  infos: HashMap<String, ImageInfo>,
  image: RgbImage,
}

impl Texts {
  pub fn build(texts: Vec<String>) -> Self {
    let mut images: Vec<(Image, String)> = vec![];
    for text in texts {
      images.push((Image::from_text(&text), text));
    }
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    for image in &images {
      width = width.max(image.0.image.width());
      height += image.0.image.height();
    }
    let mut image = RgbImage::new(width, height);
    let mut infos = HashMap::new();
    let mut y = 0;
    for img in &images {
      image.copy_from(&img.0.image, 0, y).unwrap();
      infos.insert(img.1.clone(), ImageInfo{ rect: (0, y)});
      y += img.0.image.height();
    }
    Texts {
      infos,
      image,
    }
  }

  pub fn text(str: String) -> Texture {
    // build texture for string
  }
}
