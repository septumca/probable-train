use std::{fmt::Debug, time::{UNIX_EPOCH, SystemTime}, collections::HashMap, hash::Hash};

use macroquad::{prelude::*, rand::{ChooseRandom, srand}, telemetry::ZoneGuard};


const TILE_SIZE: f32 = 8.;
const PATTERN_SIZE: f32 = 32.;
const SCREEN_WIDTH: f32 = 1600.;
const SCREEN_HEIGHT: f32 = 960.;
const HISTORY_LENGTH: usize = 10;

type Edges = [Vec<Color>; 4];

fn window_conf() -> Conf {
  Conf {
    window_title: "Simple WFC".to_owned(),
    window_width: SCREEN_WIDTH as i32,
    window_height: SCREEN_HEIGHT as i32,
    ..Default::default()
  }
}

fn patterns_are_equal(pa: &Pattern, pb: &Pattern) -> bool {
  pa.texture_index == pb.texture_index
}

fn xy_from_index(index: usize, width: usize) -> (usize, usize) {
  (index % width, index / width)
}

fn index_from_xy(x: usize, y: usize, width: usize) -> usize {
  x + y * width
}

fn image_to_textures(image: &Image, step: usize) -> (Vec<Texture2D>, Vec<u32>) {
  let mut textures = vec![];
  let mut weights = vec![];
  if image.width() % step != 0 {
    return (textures, weights)
  }

  let mut x = 0;
  let mut y = 0;
  while x < image.width() && y < image.height() {
    for rot in 0..4 {
      let img = image.sub_image(Rect::new(x as f32, y as f32, step as f32, step as f32));
      let img = rotate_image(&img, rot);
      let tex = Texture2D::from_image(&img);
      tex.set_filter(FilterMode::Nearest);

      let existing_texture_index = textures.iter().enumerate().find_map(|(i, t)| {
        if t.get_texture_data().get_image_data() == tex.get_texture_data().get_image_data() {
          Some(i)
        } else {
          None
        }
      });

      if let Some(i) = existing_texture_index {
        if rot == 0 {
          weights[i] += 1;
        }
      } else {
        textures.push(tex);
        weights.push(1);
      };
    }
    x += step;
    if x >= image.width() {
      x = 0;
      y += step;
    }
  }
  (textures, weights)
}

fn color_to_slice(c: &Color) -> [u8; 4] {
  [(c.r * 255.) as u8, (c.g * 255.) as u8, (c.b * 255.) as u8, (c.a * 255.) as u8]
}

#[derive(Hash, PartialEq, Eq)]
struct VecU8(Vec<[u8; 4]>);

fn get_edges_for_textures(tex: &[Texture2D]) -> Vec<[usize; 4]> {
  let mut idx = 0;
  let mut edges_store : HashMap<VecU8, usize> = HashMap::new();
  let mut texture_edges: Vec<[usize; 4]> = vec![[130; 4]; tex.len() as usize];

  for (ti, t) in tex.iter().enumerate() {
    let mut edge_indexes = vec![120; 4];

    for (side_idx, e) in get_edge_colors(t).iter().enumerate() {
      let c_bytes = VecU8(e.into_iter().map(|c| color_to_slice(c)).collect());
      if let Some(i) = edges_store.get(&c_bytes) {
        edge_indexes[side_idx] = i.clone();
      } else {
        edge_indexes[side_idx] = idx;
        edges_store.insert(c_bytes, idx);
        idx += 1;
      }
    }

    texture_edges[ti] = [edge_indexes[0], edge_indexes[1], edge_indexes[2], edge_indexes[3]];
  }

  texture_edges
}

fn get_edge_colors(tex: &Texture2D) -> Edges {
  let img = tex.get_texture_data();

  let mut top = vec![];
  let mut left = vec![];
  let mut bottom = vec![];
  let mut right = vec![];

  for x in 0..img.width() {
    top.push(img.get_pixel(x as u32, 0));
    bottom.push(img.get_pixel(x as u32, img.height() as u32 - 1));
  }
  for y in 0..img.height() {
    right.push(img.get_pixel(img.width() as u32 - 1, y as u32));
    left.push(img.get_pixel(0, y as u32));
  }

  [top, right, bottom, left]
}

#[derive(Clone, PartialEq)]
struct Pattern  {
  texture_index: usize,
  edges: [usize; 4],
  weight: u32,
}

impl Debug for Pattern {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Pattern: edges: {:?}", self.edges)
  }
}

impl Pattern {
  pub fn new(texture_index: usize, texture_edges: &Vec<[usize; 4]>, weight: u32) -> Self {

    Self {
      texture_index,
      edges: texture_edges[texture_index],
      weight,
    }
  }

  // pub fn draw(&self, x: f32, y: f32, res: &[Texture2D]) {
  //   draw_texture_ex(
  //     res[self.texture_index],
  //     x, y,
  //     WHITE,
  //     DrawTextureParams {
  //       dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
  //       ..Default::default()
  //     },
  //   );
  // }
}

#[derive(Clone, Debug)]
struct MapTile {
  collapsed: Option<Pattern>,
  options: Vec<Pattern>,
}

impl MapTile {
  pub fn new(options: Vec<Pattern>) -> Self {
    Self {
      collapsed: None,
      options,
    }
  }

  pub fn choose_pattern(&mut self) -> Pattern {
    let total_weights = self.options.iter().fold(0, |acc, p| acc + p.weight);
    let mut target_weight = rand::gen_range(0, total_weights+1) as i32;
    self.options
      .iter()
      .find(|p| {
        target_weight -= p.weight as i32;
        target_weight <= 0
      })
      .unwrap()
      .clone()
  }

  // pub fn draw(&self, x: f32, y: f32, res: &[Texture2D]) {
  //   if let Some(collapsed) = &self.collapsed {
  //     collapsed.draw(x, y, res);
  //   } else {
  //     let _font_size = TILE_SIZE;
  //     if self.options.len() == 0 {
  //       draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, RED);
  //     }
  //     // draw_text(&format!("{}", self.options.len()), x, y + font_size, font_size, BLACK);
  //   }
  // }
}

#[derive(Clone)]
struct Map {
  width: usize,
  height: usize,
  tiles: Vec<MapTile>,
  image: Image,
  texture: Texture2D,
  history: Vec<(Vec<(MapTile, usize)>, Pattern)>
}

impl Map {
  pub fn new(options: &Vec<Pattern>, width: usize, height: usize) -> Self {
    let mut tiles = vec![];
    for _ in 0..width * height{
      tiles.push(MapTile::new(options.clone()));
    }
    let image = Image::gen_image_color((width * PATTERN_SIZE as usize) as u16, (height * PATTERN_SIZE as usize) as u16, DARKGRAY);

    Self { width, height, tiles, history: vec![], image: image.clone(), texture: Texture2D::from_image(&image)  }
  }

  pub fn reset(&mut self, options: &Vec<Pattern>) {
    for t in &mut self.tiles {
      t.options = options.clone();
      t.collapsed = None;
    }
    self.history.clear();
  }

  // pub fn draw(&self, res: &[Texture2D]) {
  //   let _z = ZoneGuard::new("draw");
  //   for i in 0..self.tiles.len() {
  //     let (x, y) = xy_from_index(i, self.width);
  //     self.tiles[i].draw(x as f32 *  TILE_SIZE, y as f32 *  TILE_SIZE, res);
  //   }
  // }

  pub fn draw_image(&self) {
    draw_texture_ex(
      self.texture,
      0., 0.,
      WHITE,
      DrawTextureParams {
        dest_size: Some(vec2(self.width as f32 * TILE_SIZE, self.height as f32 * TILE_SIZE)),
        ..Default::default()
      },
    );
  }

  pub fn compare(&mut self, pattern: &Pattern, x: usize, y: usize, edge_size_i: usize, edge_size_j: usize) {
    let _z = ZoneGuard::new("compare");
    let j = index_from_xy(x, y, self.width);
    let edge = pattern.edges[edge_size_i].clone();
    self.tiles[j].options.retain(|p| p.edges[edge_size_j] == edge);
  }

  pub fn unwind(&mut self) {
    let _z = ZoneGuard::new("unwind");
    let (mut map_tiles, pattern) = self.history.pop().unwrap();
    map_tiles[0].0.options.retain(|p| !patterns_are_equal(p, &pattern));

    for (t, i) in map_tiles {
      self.tiles[i] = t;
    }
  }

  pub fn get_surrounding_indexes(&self, x: usize, y: usize) -> Vec<usize> {
    let mut idx = vec![];
    idx.push(index_from_xy(x, y, self.width));
    //top
    if y != 0 {
      idx.push(index_from_xy(x, y - 1, self.width));
    }
    //left
    if x != self.width - 1 {
      idx.push(index_from_xy(x + 1, y, self.width));
    }
    //bottom
    if y != self.height - 1 {
      idx.push(index_from_xy(x, y + 1, self.width));
    }
    //right
    if x != 0 {
      idx.push(index_from_xy(x - 1, y, self.width));
    }
    //top-left
    if y != 0 && x != 0 {
      idx.push(index_from_xy(x - 1, y - 1, self.width));
    }
    //top-right
    if y != 0 && x != self.width - 1 {
      idx.push(index_from_xy(x + 1, y - 1, self.width));
    }
    //bottom-left
    if y != self.height - 1 && x != 0 {
      idx.push(index_from_xy(x - 1, y + 1, self.width));
    }
    //bottom-right
    if y != self.height - 1 && x != self.width - 1 {
      idx.push(index_from_xy(x + 1, y + 1, self.width));
    }
    idx
  }

  pub fn apply_collapse(&mut self, pattern: &Pattern, x: usize, y: usize, res: &[Texture2D]) {
    let image = res[pattern.clone().texture_index].get_texture_data();
    for ix in 0..(PATTERN_SIZE as u32) {
      for iy in 0..(PATTERN_SIZE as u32) {
        self.image.set_pixel((x * PATTERN_SIZE as usize) as u32 + ix, (y * PATTERN_SIZE as usize) as u32 + iy, image.get_pixel(ix, iy));
      }
    }
    self.texture.update(&self.image);
  }

  pub fn step(&mut self, res: &[Texture2D]) -> bool {
    let _z = ZoneGuard::new("step");
    if let Some((x, y)) = self.lowest_entropy_tile() {
      let map_tiles = self.get_surrounding_indexes(x, y).iter().map(|i| (self.tiles[*i].clone(), *i)).collect();
      if let Some(pattern) = self.collapse(x, y) {
        let _z = ZoneGuard::new("step");
        self.apply_collapse(&pattern, x, y, res);

        self.history.push((map_tiles, pattern));
        if self.history.len() - 1 == HISTORY_LENGTH {
          self.history.remove(0);
        }

        return true;
      } else if self.history.len() > 0 {
        self.unwind();
        return true;
        // return self.step();
      }
    }
    false
  }

  pub fn lowest_entropy_tile(&self) -> Option<(usize, usize)> {
    let _z = ZoneGuard::new("lowest_entropy_tile");
    let mut lowest_entropy: Vec<(usize, usize)> = vec![];
    let mut lowest_entropy_value = usize::MAX;
    for i in 0..self.tiles.len() {
      let t = &self.tiles[i];
      let (x, y) = xy_from_index(i, self.width);
      if t.collapsed.is_none() {
        let entropy_value = t.options.len();
        if entropy_value < lowest_entropy_value {
          lowest_entropy_value = entropy_value;
          lowest_entropy.clear();
          lowest_entropy.push((x, y));
        } else if entropy_value == lowest_entropy_value {
          lowest_entropy.push((x, y));
        }
      }
    }

    if let Some((x, y)) = lowest_entropy.choose() {
      Some((*x, *y))
    } else {
      None
    }
  }

  pub fn collapse(&mut self, x: usize, y: usize) -> Option<Pattern> {
    let _z = ZoneGuard::new("collapse");
    // let (x, y) = self.lowest_entropy_tile();
    let i = index_from_xy(x, y, self.width);
    if self.tiles[i].options.len() == 0 {
      return None;
    }

    // let pattern = self.tiles[i].options.choose().unwrap().clone();
    let pattern = self.tiles[i].choose_pattern();

    //top
    if y != 0 {
      self.compare(&pattern, x, y - 1, 0, 2);
    }
    //left
    if x != self.width - 1 {
      self.compare(&pattern, x + 1, y, 1, 3);
    }
    //bottom
    if y != self.height - 1 {
      self.compare(&pattern, x, y + 1, 2, 0);
    }
    //right
    if x != 0 {
      self.compare(&pattern, x - 1, y, 3, 1);
    }

    self.tiles[i].collapsed = Some(pattern.clone());

    Some(pattern)
  }
}

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

fn draw_patterns(resources: &[Texture2D], patterns: &[Pattern]) {
  let mut x = PATTERN_SIZE;
  let mut y = PATTERN_SIZE;
  for p in patterns {
    // draw_text(&format!("{}", texture_edges[i][0]), x, y - TILE_SIZE / 4., 2. * TILE_SIZE, BLACK);
    // draw_text(&format!("{}", texture_edges[i][1]), x + TILE_SIZE, y + TILE_SIZE / 2., 2. * TILE_SIZE, BLACK);
    // draw_text(&format!("{}", texture_edges[i][2]), x, y + 1.75 * TILE_SIZE, 2. * TILE_SIZE, BLACK);
    // draw_text(&format!("{}", texture_edges[i][3]), x - TILE_SIZE / 2., y + TILE_SIZE / 2., 2. * TILE_SIZE, BLACK);
    draw_texture(resources[p.texture_index], x, y, WHITE);
    draw_text(&format!("{}", p.weight), x, y, PATTERN_SIZE, BLACK);
    x += PATTERN_SIZE + 2. * PATTERN_SIZE;
    if x > SCREEN_WIDTH {
      x = PATTERN_SIZE;
      y += PATTERN_SIZE + 2. * PATTERN_SIZE;
    }
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  set_pc_assets_folder("assets");
  let image = load_texture("template-1.png").await.expect("tiles.png should be loaded").get_texture_data();
  let (resources, weights) = image_to_textures(&image, PATTERN_SIZE as usize);
  let texture_edges = get_edges_for_textures(&resources);
  let options: Vec<Pattern> = resources.clone().iter().enumerate().map(|(i, _tex)| Pattern::new(i, &texture_edges, weights[i])).collect();
  let mut map = Map::new(&options, (SCREEN_WIDTH / TILE_SIZE) as usize, (SCREEN_HEIGHT / TILE_SIZE) as usize);
  let mut play = false;
  let mut started = false;
  let mut can_continue = true;
  let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
  let seed = since_the_epoch.as_secs();
  srand(seed);

  loop {
    clear_background(DARKGRAY);

    if can_continue && play {
      // can_continue = map.collapse().is_some();
      can_continue = map.step(&resources);
    }

    if is_key_released(KeyCode::Enter) {
      play = !play;
    }
    if can_continue && !play && is_key_released(KeyCode::Space) {
      // can_continue = map.collapse().is_some();
      can_continue = map.step(&resources);
    }
    if !can_continue && is_key_released(KeyCode::Space) {
      // export_map(&map, &resources);
      map.reset(&options);
      can_continue = true;
    }
    if !can_continue && is_key_released(KeyCode::E) {
      map.image.export_png(&format!("export-{}", seed));
    }

    if !started {
      draw_patterns(&resources, &options);
      started = map.tiles.iter().any(|t| t.collapsed.is_some())
    } else {
      // map.draw(&resources);
      map.draw_image();
      // map.draw_by_res_index(&resources);
    }

    // draw_text(&format!("paused: {}, running: {}, history length: {}", !play, can_continue, map.history.len()), 2., 32., 30., WHITE);
    #[cfg(debug_assertions)]
    macroquad_profiler::profiler(Default::default());

    next_frame().await
  }
}
