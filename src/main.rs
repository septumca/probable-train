use std::{collections::{HashMap, HashSet}, time::{UNIX_EPOCH, SystemTime}, iter::FromIterator};

use macroquad::{prelude::*, rand::{ChooseRandom, srand}, telemetry::ZoneGuard};

const TILE_SIZE: f32 = 8.;
const SCREEN_WIDTH: f32 = 1600.;
const SCREEN_HEIGHT: f32 = 800.;
const N: isize = 3;
const GRID_OFFSET: f32 = 0.;
const HISTORY_LENGHT: usize = 10;
const N_INDEXES: [(isize, isize); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
const WRAP_W: bool = true;
const WRAP_H: bool = true;
const ROTATE: bool = false;

fn window_conf() -> Conf {
  Conf {
    window_title: "WFC".to_owned(),
    window_width: SCREEN_WIDTH as i32,
    window_height: SCREEN_HEIGHT as i32,
    ..Default::default()
  }
}


fn xy_from_index(index: usize, width: usize) -> (usize, usize) {
  (index % width, index / width)
}

fn index_from_xy(x: usize, y: usize, width: usize) -> usize {
  x + y * width
}

// fn color_to_slice(c: &Color) -> [u8; 4] {
//   [(c.r * 255.) as u8, (c.g * 255.) as u8, (c.b * 255.) as u8, (c.a * 255.) as u8]
// }

fn rotate_image(image: &Image, rot: usize) -> Image {
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

fn extract_patterns_as_images(image: &Image, n: isize) -> Vec<Image> {
  let mut patterns: Vec<Image> = vec![];

  let width = if WRAP_W { image.width } else { image.width - n as u16 };
  let height = if WRAP_H { image.height } else { image.height - n as u16 };

  for x in 0..width {
    for y in 0..height {
      let pattern_image = get_pattern_image(image, n, x, y);
      if ROTATE {
        for rot in 1..4 {
          let rotated_image = rotate_image(&pattern_image, rot);
          if patterns.iter().all(|p| p.get_image_data() != rotated_image.get_image_data()) {
            patterns.push(rotated_image);
          }
        }
      }
      if patterns.iter().all(|p| p.get_image_data() != pattern_image.get_image_data()) {
        patterns.push(pattern_image);
      }
    }
  }

  patterns
}

type OverlapData = Vec<HashMap<(isize, isize), Vec<usize>>>;

fn create_overlap_patterns(patterns: &[Image], n: isize) -> OverlapData  {
  let mut texture_overlaps = vec![];
  for img in patterns {
    let mut overlap_pattern = HashMap::new();
    for (ox, oy) in N_INDEXES {
      let mut valid_tex_idxes = vec![];
      for (overlap_idx, overlap_img) in patterns.iter().enumerate() {
        let mut valid = true;
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
      overlap_pattern.insert((ox, oy), valid_tex_idxes);
    }
    texture_overlaps.push(overlap_pattern);
  }
  texture_overlaps
}

fn draw_patterns(y_offset: f32, pattern_indexes: &[usize], patterns: &[Texture2D], text: &str) {
  let start_x = 100. + 10. * TILE_SIZE;
  let mut x = start_x;
  let mut y = y_offset;
  draw_text(text, x - 10., y - 50., 30., WHITE);
  for tex_id in pattern_indexes {
    let tex = patterns[*tex_id];

    draw_rectangle_lines(x - 10., y - 10., TILE_SIZE + 20., TILE_SIZE + 20., 5., YELLOW);
    draw_texture_ex(
      tex, x, y, WHITE,
      DrawTextureParams {
        dest_size: Some(Vec2::splat(TILE_SIZE)),
        ..Default::default()
      }
    );
    draw_rectangle_lines(x, y, TILE_SIZE / N as f32, TILE_SIZE / N as f32, 3., RED);
    draw_text(&format!("{}", tex_id), x - 10., y - 14., TILE_SIZE, WHITE);
    x += TILE_SIZE + 30.;
    if x + 30. >= SCREEN_WIDTH {
      x = start_x;
      y += TILE_SIZE + 50.;
    }
  }
}


struct Grid {
  width: usize,
  height: usize,
  cells: Vec<Option<usize>>,
  options: Vec<Vec<usize>>,
  entropy: Vec<usize>,
  overlaps: OverlapData,
  colors: Vec<Color>,
  history: Vec<((usize, usize), HashSet<usize>, Vec<Vec<usize>>)>,
}

impl Grid {
  pub fn new(width: usize, height: usize, patterns_length: usize, overlaps: &OverlapData, colors: &Vec<Color>) -> Self {
    Self {
      width,
      height,
      cells: vec![None; width * height],
      options: vec![(0..patterns_length).collect(); width * height],
      entropy: vec![patterns_length; width * height],
      overlaps: overlaps.clone(),
      colors: colors.clone(),
      history: vec![],
    }
  }

  fn draw(&self) {
    let _z = ZoneGuard::new("draw");
    for (index, pattern) in self.cells.iter().enumerate() {
      let (x, y) = xy_from_index(index, self.width);
      let x = x as f32 * TILE_SIZE + GRID_OFFSET;
      let y = y as f32 * TILE_SIZE + GRID_OFFSET;
      if let Some(p) = pattern {
        draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, self.colors[*p]);
      }
    }
  }

  fn unwind(&mut self) {
    if let Some(((invalid_pattern, invalid_idx), updated_tiles, options)) = self.history.pop() {
      for idx in updated_tiles {
        self.options[idx] = options[idx].clone();
        self.cells[idx] = None;
      }
      self.options[invalid_idx] = options[invalid_idx].iter().filter_map(|p| if *p != invalid_pattern { Some(*p) } else { None }).collect();
    }
  }

  fn step(&mut self) {
    let _z = ZoneGuard::new("step");
    if self.is_finished() {
      return;
    }

    let entropy_index = self.observe();
    if let Some(p) = self.collapse(entropy_index) {
      let options_store = self.options.clone();
      let updated_tiles = self.propagate(entropy_index);
      if self.history.len() == HISTORY_LENGHT {
        self.history.remove(0);
      }
      self.history.push(((p, entropy_index), updated_tiles, options_store));
    } else {
      self.unwind();
    }
  }

  fn observe(&self) -> usize {
    let _z = ZoneGuard::new("observe");
    let mut lowest_entropy: Vec<usize> = vec![];
    let mut lowest_entropy_value = usize::MAX;
    for i in 0..self.options.len() {
      if self.cells[i].is_some() {
        continue;
      }
      let entropy_value = self.options[i].len();
      if entropy_value < lowest_entropy_value {
        lowest_entropy_value = entropy_value;
        lowest_entropy.clear();
        lowest_entropy.push(i);
      } else if entropy_value == lowest_entropy_value {
        lowest_entropy.push(i);
      }
    }

    *lowest_entropy.choose().unwrap()
  }

  fn is_finished(&self) -> bool {
    self.cells.iter().all(|v| v.is_some())
  }

  fn collapse(&mut self, idx: usize) -> Option<usize> {
    let _z = ZoneGuard::new("collapse");
    self.options[idx].choose().and_then(|p| {
     self.cells[idx] = Some(*p);
     Some(*p)
    })
  }

  fn propagate(&mut self, idx: usize) -> HashSet<usize> {
    let _z = ZoneGuard::new("propagate");
    let mut stack = vec![idx];
    let mut visited_tiles: HashSet<usize> = HashSet::new();

    while let Some(idx) = stack.pop() {
      if visited_tiles.contains(&idx) {
        continue;
      }
      visited_tiles.insert(idx);
      if self.options[idx].len() == 1 {
        self.collapse(idx);
      }
      // println!("==================================================");
      // println!("processing index: {}", idx);
      // let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)];
      for (dx, dy) in N_INDEXES {
        let (x, y) = xy_from_index(idx, self.width);
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        if nx < 0 || nx >= self.width as isize || ny < 0 || ny >= self.height as isize {
          continue;
        }
        let neighbour_idx = index_from_xy(nx as usize, ny as usize, self.width);
        if self.cells[neighbour_idx].is_some() {
          continue;
        }
        let overlaps: Vec<usize> = if let Some(pattern) = self.cells[idx] {
          self.overlaps[pattern].get(&(dx, dy)).unwrap().clone()
        } else {
          let vec1: Vec<usize> = self.options[idx].iter().flat_map(|opt| self.overlaps[*opt].get(&(dx, dy)).unwrap().clone()).collect();
          let hs = HashSet::<_>::from_iter(vec1);
          let vec2: Vec<usize> = hs.into_iter().collect();
          vec2
        };

        // println!("{},{} => {},{} => valid patterns: {:?}", x, y, nx, ny, overlaps);
        let options_before = self.options[neighbour_idx].len();
        self.options[neighbour_idx].retain(|p| overlaps.contains(p));
        let options_now = self.options[neighbour_idx].len();
        // println!("old: {options_before}, new: {options_now}");

        if options_now < options_before {
          // println!("adding idx {neighbour_idx} for processing");
          stack.insert(0, neighbour_idx);
          // self.entropy[neighbour_idx] = options_now;
        }
      }
    }

    visited_tiles
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  set_pc_assets_folder("assets");
  let image = load_texture("pat-bricks-1px.png").await.expect("image should be loaded").get_texture_data();
  let images = extract_patterns_as_images(&image, N);
  println!("extracted {} image patterns", images.len());
  let pattern_colors: Vec<Color> = images.iter().map(|t| t.get_pixel(0, 0)).collect();
  let overlaps = create_overlap_patterns(&images, N);
  let mut play = true;
  let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
  let seed = since_the_epoch.as_secs();
  srand(seed);

  let width = (SCREEN_WIDTH / TILE_SIZE) as usize;
  let height = (SCREEN_HEIGHT / TILE_SIZE) as usize;
  let mut grid = Grid::new(
    width,
    height,
    pattern_colors.len(),
    &overlaps,
    &pattern_colors
  );

  loop {
    clear_background(DARKGRAY);

    if is_key_released(KeyCode::R) {
      grid = Grid::new(
        width,
        height,
        pattern_colors.len(),
        &overlaps,
        &pattern_colors
      );
    }
    if is_key_released(KeyCode::P) {
      play = !play;
    }
    if is_key_released(KeyCode::Space) {
      grid.step();
    }
    if play {
      grid.step();
    }
    grid.draw();

    #[cfg(debug_assertions)]
    {
      draw_text(&format!("running: {}, history: {}", play, grid.history.len()), 2., 32., 30., WHITE);
      macroquad_profiler::profiler(Default::default());
    }

    next_frame().await
  }
}
