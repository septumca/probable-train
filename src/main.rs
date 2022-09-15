use macroquad::{prelude::*};

const TILE_SIZE: f32 = 16.;
const SCREEN_WIDTH: f32 = 1600.;
const SCREEN_HEIGHT: f32 = 800.;

fn window_conf() -> Conf {
  Conf {
    window_title: "Simple WFC".to_owned(),
    window_width: SCREEN_WIDTH as i32,
    window_height: SCREEN_HEIGHT as i32,
    ..Default::default()
  }
}


// fn xy_from_index(index: usize, width: usize) -> (usize, usize) {
//   (index % width, index / width)
// }

// fn index_from_xy(x: usize, y: usize, width: usize) -> usize {
//   x + y * width
// }

// fn color_to_slice(c: &Color) -> [u8; 4] {
//   [(c.r * 255.) as u8, (c.g * 255.) as u8, (c.b * 255.) as u8, (c.a * 255.) as u8]
// }

fn rotate_image(image: &Image, rot: usize) -> Image {
  let mut new_image = image.clone();
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

fn extract_patterns_as_textures(image: &Image, n: isize) -> Vec<Texture2D> {
  let tiles_count = image.width / n as u16 * image.height / n as u16;
  let mut tile_progress = 0;
  let mut patterns: Vec<Image> = vec![];

  let mut x: usize = 0;
  let mut y: usize = 0;
  while tile_progress < tiles_count {
    let pattern_image = image.sub_image(Rect::new(x as f32, y as f32, n as f32, n as f32));
    for rot in 0..4 {
      let pattern_image = rotate_image(&pattern_image, rot);
      if patterns.iter().all(|p| p.get_image_data() != pattern_image.get_image_data()) {
        patterns.push(pattern_image);
      }
    }

    x += n as usize;
    if x >= image.width() {
      x = 0;
      y += n as usize;
    }
    tile_progress += 1;
  }

  patterns.into_iter()
    .map(|img| {
      let tex = Texture2D::from_image(&img);
      tex.set_filter(FilterMode::Nearest);
      tex
    })
    .collect()
}

fn overlap_indexes(n: isize) -> Vec<(isize, isize)> {
  let mut idxes = vec![];
  for x in -n+1..n {
    for y in -n+1..n {
      if x != 0 || y != 0 {
        idxes.push((x, y))
      }
    }
  }
  idxes
}

type OverlapData = Vec<Vec<((isize, isize), Vec<usize>)>>;

fn create_overlap_patterns(textures: &[Texture2D], n: isize) -> OverlapData  {
  let overlap_idxes = overlap_indexes(n);
  let mut texture_overlaps = vec![];
  for tex in textures {
    let img = tex.get_texture_data();
    let mut overlap_pattern = vec![];
    for (ox, oy) in &overlap_idxes {
      let mut valid_tex_idxes = vec![];
      for (overlap_idx, overlap_tex) in textures.iter().enumerate() {
        let mut valid = true;
        let overlap_img = overlap_tex.get_texture_data();
        'coords: for x in 0..n {
          for y in 0..n {
            let tx = ox + x;
            let ty = oy + y;
            if tx >= 0 && tx < n && ty >= 0 && ty < n {
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
      overlap_pattern.push(((*ox, *oy), valid_tex_idxes));
    }
    texture_overlaps.push(overlap_pattern);
  }
  texture_overlaps
}

fn draw_patterns(patterns: &[Texture2D], overlaps: &OverlapData) {
  let mut x = 120.;
  let mut y = 120.;
  for (tex_id, tex) in patterns.iter().enumerate() {
    let tex_overlap = &overlaps[tex_id];
    for ((ox, oy), overlap_idxes) in tex_overlap {
      let (dx, dy) = (x + 80. * *ox as f32 - 10., y + 80. * *oy as f32- 10.);
      draw_rectangle_lines(dx, dy, TILE_SIZE + 50., TILE_SIZE + 50., 5., YELLOW);
      let mut tdx = 4.;
      let mut tdy = TILE_SIZE;
      for overlap_idx in overlap_idxes {
        draw_text(&format!("{}", overlap_idx), dx + tdx, dy + tdy, TILE_SIZE, WHITE);
        tdx += TILE_SIZE;
        if tdx >= TILE_SIZE + 40. {
          tdx = 4.;
          tdy += TILE_SIZE;
        }
      }
    }
    draw_rectangle_lines(x - 10., y - 10., TILE_SIZE + 20., TILE_SIZE + 20., 5., YELLOW);
    draw_texture_ex(
      *tex, x, y, WHITE,
      DrawTextureParams {
        dest_size: Some(Vec2::splat(TILE_SIZE)),
        ..Default::default()
      }
    );
    draw_text(&format!("{}", tex_id), x - 10., y - 14., TILE_SIZE, WHITE);
    draw_rectangle_lines(x - 100., y - 100., TILE_SIZE + 230., TILE_SIZE + 230., 5., ORANGE);
    x += 260.;
    if x >= SCREEN_WIDTH {
      x = 120.;
      y += 260.;
    }
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  set_pc_assets_folder("assets");
  let image = load_texture("4x4.png").await.expect("image should be loaded").get_texture_data();
  let n = 2;
  let textures = extract_patterns_as_textures(&image, n);
  let overlaps = create_overlap_patterns(&textures, n);
  // let mut play = false;

  loop {
    clear_background(DARKGRAY);

    draw_patterns(&textures, &overlaps);

    #[cfg(debug_assertions)]
    {
      // draw_text(&format!("running: {}", play), 2., 32., 30., WHITE);
      // macroquad_profiler::profiler(Default::default());
    }

    next_frame().await
  }
}
