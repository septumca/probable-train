use std::{fmt::Debug, time::{UNIX_EPOCH, SystemTime}, collections::HashMap, hash::Hash};

use macroquad::{prelude::*, rand::{ChooseRandom, srand}, telemetry::ZoneGuard};


const TILE_SIZE: f32 = 8.;
const PATTERN_SIZE: f32 = 32.;
const SCREEN_WIDTH: f32 = 1600.;
const SCREEN_HEIGHT: f32 = 800.;

const EDGES_INDEXES: [(usize, usize); 4] = [(0, 2), (1, 3), (2, 0), (3, 1)];


type Edges = [Vec<Color>; 4];

fn window_conf() -> Conf {
  Conf {
    window_title: "Simple WFC".to_owned(),
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
}

#[derive(Clone)]
struct Map {
  width: usize,
  height: usize,
  tiles: Vec<MapTile>,
  image: Image,
  texture: Texture2D,
}

impl Map {
  pub fn new(options: &Vec<Pattern>, width: usize, height: usize) -> Self {
    let mut tiles = vec![];
    for _ in 0..width * height{
      tiles.push(MapTile::new(options.clone()));
    }
    let image = Image::gen_image_color((width * PATTERN_SIZE as usize) as u16, (height * PATTERN_SIZE as usize) as u16, DARKGRAY);

    Self { width, height, tiles, image: image.clone(), texture: Texture2D::from_image(&image)  }
  }

  pub fn reset(&mut self, options: &Vec<Pattern>) {
    for t in &mut self.tiles {
      t.options = options.clone();
      t.collapsed = None;
    }
    self.image = Image::gen_image_color((self.width * PATTERN_SIZE as usize) as u16, (self.height * PATTERN_SIZE as usize) as u16, DARKGRAY);
    self.texture = Texture2D::from_image(&self.image);
  }

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
    #[cfg(debug_assertions)]
    for i in 0..self.width * self.height {
      let (x, y) = xy_from_index(i, self.width);
      let options_len = self.tiles[i].options.len();
      draw_text(&format!("{}", options_len), x as f32 * TILE_SIZE, TILE_SIZE + y as f32 * TILE_SIZE, TILE_SIZE, BLACK);
    }
  }

  pub fn apply_collapse(&mut self, x: usize, y: usize, res: &[Texture2D]) {
    let i = index_from_xy(x, y, self.width);
    if let Some(pattern) = self.tiles[i].collapsed.clone() {
      let image = res[pattern.clone().texture_index].get_texture_data();
      for ix in 0..(PATTERN_SIZE as u32) {
        for iy in 0..(PATTERN_SIZE as u32) {
          self.image.set_pixel((x * PATTERN_SIZE as usize) as u32 + ix, (y * PATTERN_SIZE as usize) as u32 + iy, image.get_pixel(ix, iy));
        }
      }
      self.texture.update(&self.image);
    }
  }

  pub fn step(&mut self, res: &[Texture2D]) -> bool {
    let _z = ZoneGuard::new("step");
    if let Some((x, y)) = self.lowest_entropy_tile() {
      if self.collapse(x, y) {
        let collapsed_by_propagate = self.propagate(x, y);
        self.apply_collapse(x, y, res);

        for (x, y) in collapsed_by_propagate {
          self.apply_collapse(x, y, res);
        }
        return true;
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

  pub fn get_neighbours(&self, x: usize, y: usize) -> [Option<(usize, usize)>; 4] {
    let mut result = [None, None, None, None];
    //top
    if y != 0 {
      result[0] = Some((x, y - 1));
    }
    //left
    if x != self.width - 1 {
      result[1] = Some((x + 1, y));
    }
    //bottom
    if y != self.height - 1 {
      result[2] = Some((x, y + 1));
    }
    //right
    if x != 0 {
      result[3] = Some((x - 1, y));
    }
    result
  }

  pub fn propagate(&mut self, x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut collapsed = vec![];
    let mut stack = vec![(x, y)];

    while stack.len() > 0 {
      let (x, y) = stack.remove(0);
      let source_idx = index_from_xy(x, y, self.width);
      let patterns = self.tiles[source_idx].collapsed.clone()
        .map(|p| vec![p])
        .unwrap_or(self.tiles[source_idx].options.clone());

      for (edge_idx, pos) in self.get_neighbours(x, y).into_iter().enumerate() {
        if let Some((tx, ty)) = pos {
          let target_idx = index_from_xy(tx, ty, self.width);
          if self.tiles[target_idx].collapsed.is_some() {
            continue;
          }

          let edge_idxes = EDGES_INDEXES[edge_idx];
          let options_before = self.tiles[target_idx].options.len();
          let edges: Vec<usize> = patterns.iter().map(|p| p.edges[edge_idxes.0]).collect();
          self.tiles[target_idx].options.retain(|p| edges.contains(&p.edges[edge_idxes.1]));

          if self.tiles[target_idx].options.len() < options_before {
            stack.push((tx, ty));
          }

          if self.tiles[target_idx].options.len() == 1 {
            self.tiles[target_idx].collapsed = Some(self.tiles[target_idx].options[0].clone());
            collapsed.push((tx, ty));
          }
        }
      }
    }

    collapsed
  }

  pub fn collapse(&mut self, x: usize, y: usize) -> bool {
    let i = index_from_xy(x, y, self.width);
    if self.tiles[i].options.len() == 0 {
      return false;
    }

    self.tiles[i].collapsed = Some(self.tiles[i].choose_pattern());
    true
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
  let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
  let seed = since_the_epoch.as_secs();
  srand(seed);

  loop {
    clear_background(DARKGRAY);

    if play {
      map.step(&resources);
    }

    if is_key_released(KeyCode::Enter) {
      play = !play;
    }
    if !play && is_key_released(KeyCode::Space) {
      play = !map.step(&resources);
    }
    if is_key_released(KeyCode::R) {
      map.reset(&options);
      play = true;
    }
    if !play && is_key_released(KeyCode::E) {
      map.image.export_png(&format!("export-{}", seed));
    }

    if !started {
      draw_patterns(&resources, &options);
      started = map.tiles.iter().any(|t| t.collapsed.is_some())
    } else {
      map.draw_image();
    }

    #[cfg(debug_assertions)]
    {
      draw_text(&format!("running: {}", play), 2., 32., 30., WHITE);
      macroquad_profiler::profiler(Default::default());
    }

    next_frame().await
  }
}
