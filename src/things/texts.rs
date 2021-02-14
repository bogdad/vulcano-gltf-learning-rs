extern crate ab_glyph;

use ab_glyph::{point, Font, FontRef, Glyph};
use image::{GenericImage, ImageFormat, Rgb, RgbImage};

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
      let glyph: Glyph = font
        .glyph_id(chr)
        .with_scale_and_position(24.0, point(0.0, 0.0));
      let q = font.outline_glyph(glyph).unwrap();
      //println!("yyyyyyyyyy {:?}", q.px_bounds());
      x += q.px_bounds().max.x - q.px_bounds().min.x;
      if y <= q.px_bounds().max.y - q.px_bounds().min.y {
        y = q.px_bounds().max.y - q.px_bounds().min.y;
      }
    }
    let w = x as u32;
    let h = y as u32;
    //println!("xxxxxxxxxx {} {}", w, h);
    let mut image = RgbImage::new(w, h);
    x = 0.0;
    for chr in text.chars() {
      let glyph: Glyph = font
        .glyph_id(chr)
        .with_scale_and_position(24.0, point(0.0, 0.0));
      let q = font.outline_glyph(glyph).unwrap();
      q.draw(|xx, yy, c| {
        //println!("zzzzzzzzzz {:?}", (xx, yy, c));
        image.put_pixel(x as u32 + xx, yy, Rgb([(255.0 * c) as u8, 0, 0]))
      });
      x += q.px_bounds().max.x - q.px_bounds().min.x;
    }

    Image { image }
  }
}

pub struct ImageInfo {
  pub min: (u32, u32),
  pub max: (u32, u32),
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
    let mut image = RgbImage::new(2 * width, 2 * height);
    let mut infos = HashMap::new();
    let mut y = 0;
    for img in &images {
      let y_to_put = img.0.image.height();
      image.copy_from(&img.0.image, 0, y + y_to_put).unwrap();
      infos.insert(
        img.1.clone(),
        ImageInfo {
          min: (0, y),
          max: (width * 2, y + 2 * img.0.image.height()),
        },
      );
      y += 2 * img.0.image.height();
    }
    image
      .save_with_format("./all.png", ImageFormat::Png)
      .unwrap();
    Texts { infos, image }
  }

  pub fn texture(&self) -> RgbImage {
    self.image.clone()
  }

  pub fn info(&self, text: &String) -> &ImageInfo {
    self.infos.get(text).unwrap()
  }

  pub fn size(&self) -> (u32, u32) {
    (self.image.width(), self.image.height())
  }
}
