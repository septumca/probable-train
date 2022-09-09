use std::{fmt::Debug, time::{UNIX_EPOCH, SystemTime}};

use macroquad::{prelude::*, rand::{ChooseRandom, srand}};


const TILE_SIZE: f32 = 32.;
const MAP_WIDTH: i32 = 75;
const MAP_HEIGHT: i32 = 40;
const HISTORY_LENGTH: usize = 10;

type Edges = [Vec<Color>; 4];

fn window_conf() -> Conf {
  Conf {
    window_title: "Simple WFC".to_owned(),
    window_width: TILE_SIZE as i32 * MAP_WIDTH,
    window_height: TILE_SIZE as i32 *  MAP_HEIGHT,
    ..Default::default()
  }
}

fn tiles_are_equal(ta: &Tile, tb: &Tile) -> bool {
  ta.texture_index == tb.texture_index
}

fn edges_are_equal<T: PartialEq>(edge_a: &[T], edge_b: &[T]) -> bool {
  if edge_a.len() != edge_b.len() {
    return false;
  }
  let mut valid = true;

  for i in 0..edge_a.len() {
    valid = valid && edge_a[i] == edge_b[i];
    if !valid {
      break
    }
  }
  valid
}

fn xy_from_index(index: usize, width: usize) -> (usize, usize) {
  (index % width, index / width)
}

fn index_from_xy(x: usize, y: usize, width: usize) -> usize {
  x + y * width
}

fn image_to_textures(image: &Image, n: usize, step: usize) -> Vec<Texture2D> {
  let mut textures = vec![];
  if image.width() % step != 0 {
    return textures
  }

  let mut act_step = 0;
  while act_step < image.width() {
    for rot in 0..4 {
      let img = image.sub_image(Rect::new(act_step as f32, 0., n as f32, n as f32));
      let img = rotate_image(&img, rot);
      let tex = Texture2D::from_image(&img);
      tex.set_filter(FilterMode::Nearest);
      textures.push(tex);
    }
    act_step += step;
  }

  textures
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
struct Tile  {
  texture_index: usize,
  edges: Edges,
}

impl Debug for Tile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Tile: edges: {:?}", self.edges)
  }
}

impl Tile {
  pub fn new(texture_index: usize, tex: &Texture2D) -> Self {
    let edges = get_edge_colors(tex);

    Self {
      texture_index,
      edges,
    }
  }

  pub fn draw(&self, x: f32, y: f32, res: &[Texture2D]) {
    draw_texture_ex(
      res[self.texture_index],
      x, y,
      WHITE,
      DrawTextureParams {
        ..Default::default()
      },
    );
  }
}

#[derive(Clone)]
struct MapTile {
  collapsed: Option<Tile>,
  options: Vec<Tile>,
}

impl MapTile {
  pub fn new(options: Vec<Tile>) -> Self {
    Self {
      collapsed: None,
      options,
    }
  }

  pub fn draw(&self, x: f32, y: f32, res: &[Texture2D]) {
    if let Some(collapsed) = &self.collapsed {
      collapsed.draw(x, y, res);
    } else {
      let font_size = TILE_SIZE;
      if self.options.len() == 0 {
        draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, RED);
      }
      draw_text(&format!("{}", self.options.len()), x, y + font_size, font_size, BLACK);
    }
  }
}

struct Map {
  width: usize,
  height: usize,
  tiles: Vec<MapTile>,
  history: Vec<(Vec<MapTile>, usize, usize, Tile)>
}

impl Map {
  pub fn new(options: &Vec<Tile>, width: usize, height: usize) -> Self {
    let mut tiles = vec![];
    for _ in 0..width * height{
      tiles.push(MapTile::new(options.clone()));
    }

    Self { width, height, tiles, history: vec![]  }
  }

  pub fn reset(&mut self, options: &Vec<Tile>) {
    for t in &mut self.tiles {
      t.options = options.clone();
      t.collapsed = None;
    }
    self.history.clear();
  }

  pub fn draw(&self, res: &[Texture2D]) {
    for i in 0..self.tiles.len() {
      let (x, y) = xy_from_index(i, self.width);
      self.tiles[i].draw(x as f32 *  TILE_SIZE, y as f32 *  TILE_SIZE, res);
    }
  }

  pub fn compare(&mut self, tile: &Tile, x: usize, y: usize, edge_size_i: usize, edge_size_j: usize) {
    let j = index_from_xy(x, y, self.width);
    let edge = tile.edges[edge_size_i].clone();
    self.tiles[j].options.retain(|t| edges_are_equal(&t.edges[edge_size_j], &edge));
  }

  pub fn unwind(&mut self) {
    let (mut map_tiles, hx, hy, tile) = self.history.pop().unwrap();
    let hi = index_from_xy(hx, hy, self.width);
    map_tiles[hi].options.retain(|t| !tiles_are_equal(t, &tile));
    self.tiles = map_tiles;
  }

  pub fn step(&mut self) -> bool {
    if let Some((x, y)) = self.lowest_entropy_tile() {
      if let Some(tile) = self.collapse(x, y) {
        self.history.push((self.tiles.clone(), x, y, tile));
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

  pub fn collapse(&mut self, x: usize, y: usize) -> Option<Tile> {
    // let (x, y) = self.lowest_entropy_tile();
    let i = index_from_xy(x, y, self.width);
    if self.tiles[i].options.len() == 0 {
      return None;
    }

    let tile = self.tiles[i].options.choose().unwrap().clone();

    //top
    if y != 0 {
      self.compare(&tile, x, y - 1, 0, 2);
    }
    //left
    if x != self.width - 1 {
      self.compare(&tile, x + 1, y, 1, 3);
    }
    //bottom
    if y != self.height - 1 {
      self.compare(&tile, x, y + 1, 2, 0);
    }
    //right
    if x != 0 {
      self.compare(&tile, x - 1, y, 3, 1);
    }

    self.tiles[i].collapsed = Some(tile.clone());

    Some(tile)
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

#[macroquad::main(window_conf)]
async fn main() {
  set_pc_assets_folder("assets");
  let image = load_texture("tiles.png").await.expect("tiles.png should be loaded").get_texture_data();
  let resources = image_to_textures(&image, TILE_SIZE as usize, TILE_SIZE as usize);
  let options: Vec<Tile> = resources.iter().enumerate().map(|(i, tex)| Tile::new(i, tex)).collect();
  let mut map = Map::new(&options, MAP_WIDTH as usize, MAP_HEIGHT as usize);
  let mut play = false;
  let mut can_continue = true;
  let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
  srand(since_the_epoch.as_secs());

  loop {
    clear_background(DARKGRAY);

    if can_continue && play {
      // can_continue = map.collapse().is_some();
      can_continue = map.step();
    }

    if is_key_released(KeyCode::Enter) {
      play = !play;
    }
    if can_continue && !play && is_key_released(KeyCode::Space) {
      // can_continue = map.collapse().is_some();
      can_continue = map.step();
    }
    if !can_continue && is_key_released(KeyCode::Space) {
      map.reset(&options);
      can_continue = true;
    }

    map.draw(&resources);

    draw_text(&format!("paused: {}, running: {}, history length: {}", !play, can_continue, map.history.len()), 2., 32., 30., WHITE);
    #[cfg(debug_assertions)]
    macroquad_profiler::profiler(Default::default());

    next_frame().await
  }
}
