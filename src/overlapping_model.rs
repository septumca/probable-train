use std::collections::HashMap;

use macroquad::prelude::*;

use crate::{utils::{rotate_image, N_INDEXES, AdjacencyData}, Drawable, TILE_SIZE, WfcPreprocessor};

#[derive(Clone)]
pub struct ColorPattern(Vec<Color>);

impl Drawable for ColorPattern {
  fn draw(&self, x: f32, y: f32, idx: usize) {
    draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, self.0[idx]);
  }
  fn len(&self) -> usize {
    self.0.len()
  }
}

pub struct OverlappingPreprocessor {
  n: isize,
  wrap_w: bool,
  wrap_h: bool,
  rotate: bool
}

impl OverlappingPreprocessor {
  pub fn new(n: isize, wrap_w: bool, wrap_h: bool, rotate: bool) -> Self {
    Self { n, wrap_w, wrap_h, rotate }
  }
}

impl WfcPreprocessor for OverlappingPreprocessor {
  type Pattern = ColorPattern;

  fn create_patterns(&self, images: &[Image]) -> Self::Pattern {
    let patterns: Vec<Color> = images.iter().map(|i| i.get_pixel(0, 0)).collect();
    ColorPattern(patterns)
  }

  fn create_adjacency_rules(&self, images: &[Image]) -> AdjacencyData {
    let mut texture_overlaps = vec![];
    for img in images {
      let mut overlap_pattern = HashMap::new();
      for (ox, oy) in N_INDEXES {
        let mut valid_tex_idxes = vec![];
        for (overlap_idx, overlap_img) in images.iter().enumerate() {
          let mut valid = true;
          'coords: for x in 0..self.n {
            for y in 0..self.n {
              let tx = ox + x;
              let ty = oy + y;
              if tx >= 0 && tx < self.n && ty >= 0 && ty < self.n {
                valid = valid && overlap_img.get_pixel(x as u32, y as u32) == img.get_pixel(tx as u32, ty as u32);
              }
              if !valid {
                break 'coords;
              }
            }
          }
          if valid {
            valid_tex_idxes.push(overlap_idx);
          }
        }
        valid_tex_idxes.reverse();
        overlap_pattern.insert((ox, oy), valid_tex_idxes);
      }
      texture_overlaps.push(overlap_pattern);
    }
    texture_overlaps
  }

  fn extract_images(&self, image: &Image) -> Vec<Image> {
    let mut images: Vec<Image> = vec![];

    let width = if self.wrap_w { image.width } else { image.width - self.n as u16 };
    let height = if self.wrap_h { image.height } else { image.height - self.n as u16 };

    for x in 0..width {
      for y in 0..height {
        let img = get_pattern_image(image, self.n, x, y);
        if self.rotate {
          for rot in 1..4 {
            let rotated_image = rotate_image(&img, rot);
            if images.iter().all(|p| p.get_image_data() != rotated_image.get_image_data()) {
              images.push(rotated_image);
            }
          }
        }
        if images.iter().all(|p| p.get_image_data() != img.get_image_data()) {
          images.push(img);
        }
      }
    }
    images
  }
}

fn get_pattern_image(src_image: &Image, n: isize, x: u16, y: u16) -> Image {
  let mut pattern_image = Image::gen_image_color(n as u16, n as u16, WHITE);
  for px in 0..n {
    for py in 0..n {
      let color = src_image.get_pixel(((x + px as u16) % src_image.width) as u32 , ((y + py as u16) % src_image.height) as u32);
      pattern_image.set_pixel(
        px as u32,
        py as u32,
        color
      );
    }
  }
  pattern_image
}
