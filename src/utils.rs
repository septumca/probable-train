use std::collections::HashMap;

use macroquad::prelude::*;

pub const N_INDEXES: [(isize, isize); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

pub type AdjacencyData = Vec<HashMap<(isize, isize), Vec<usize>>>;

pub fn xy_from_index(index: usize, width: usize) -> (usize, usize) {
  (index % width, index / width)
}

pub fn index_from_xy(x: usize, y: usize, width: usize) -> usize {
  x + y * width
}

// fn color_to_slice(c: &Color) -> [u8; 4] {
//   [(c.r * 255.) as u8, (c.g * 255.) as u8, (c.b * 255.) as u8, (c.a * 255.) as u8]
// }

pub fn rotate_image(image: &Image, rot: usize) -> Image {
  let mut new_image = image.clone();
  if rot == 0 {
    return new_image;
  }

  for _ in 0..rot {
    let image_store = new_image.clone();
    for x in 0..new_image.width() {
      for y in 0..new_image.height() {
        let x = x as u32;
        let y = y as u32;
        // new_image.set_pixel(y, image.width() as u32 - 1 - x, image.get_pixel(x, y)); //Anti-CW
        new_image.set_pixel(image_store.height() as u32 - 1 - y, x, image_store.get_pixel(x, y)); //CW
      }
    }
  }
  new_image
}
