use std::collections::HashMap;

use macroquad::prelude::*;

use crate::TILE_SIZE;

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

pub fn draw_patterns(patterns: &[Image], y_offset: f32, text: &str) {
  let start_x = 100. + 10. * TILE_SIZE;
  let mut x = start_x;
  let mut y = y_offset;
  draw_text(text, x - 10., y - 50., 30., WHITE);
  for (idx, img) in patterns.iter().enumerate() {
    let tex = Texture2D::from_image(&img);
    draw_rectangle_lines(x - 10., y - 10., TILE_SIZE + 20., TILE_SIZE + 20., 5., YELLOW);
    draw_texture_ex(
      tex, x, y, WHITE,
      DrawTextureParams {
        dest_size: Some(Vec2::splat(TILE_SIZE)),
        ..Default::default()
      }
    );
    draw_text(&format!("{}", idx), x - 10., y - 14., TILE_SIZE, WHITE);
    x += TILE_SIZE + 30.;
    if x + 30. >= screen_width() {
      x = start_x;
      y += TILE_SIZE + 50.;
    }
  }
}
