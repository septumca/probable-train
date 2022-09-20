use std::{collections::{HashSet}, time::{UNIX_EPOCH, SystemTime}, iter::FromIterator};

use macroquad::{prelude::*, rand::{ChooseRandom, srand}, telemetry::ZoneGuard};
use overlapping_model::OverlappingPreprocessor;
use tile_model::TileProcessor;
use utils::{AdjacencyData, xy_from_index, index_from_xy, N_INDEXES};

mod utils;
mod overlapping_model;
mod tile_model;

const TILE_SIZE: f32 = 16.;
const SCREEN_WIDTH: f32 = 1600.;
const SCREEN_HEIGHT: f32 = 800.;
const GRID_OFFSET: f32 = 0.;
const HISTORY_LENGHT: usize = 20;

fn window_conf() -> Conf {
  Conf {
    window_title: "WFC".to_owned(),
    window_width: SCREEN_WIDTH as i32,
    window_height: SCREEN_HEIGHT as i32,
    ..Default::default()
  }
}

pub trait Drawable {
  fn draw(&self, x: f32, y: f32, idx: usize);
  fn len(&self) -> usize;
}

pub trait WfcPreprocessor {
  type Pattern: Drawable + Clone;
  fn extract_images(&self, image: &Image) -> Vec<Image>;
  fn create_patterns(&self, images: &[Image]) -> Self::Pattern;
  fn create_adjacency_rules(&self, images: &[Image]) -> AdjacencyData;
}

pub fn process<P: WfcPreprocessor>(processor: &P, image: &Image) -> (P::Pattern, AdjacencyData) {
  let images = processor.extract_images(&image);
  println!("extracted {} image patterns", images.len());
  let patterns = processor.create_patterns(&images);
  let adjacency_rules = processor.create_adjacency_rules(&images);

  (patterns, adjacency_rules)
}


struct Grid<P: Drawable + Clone> {
  width: usize,
  height: usize,
  cells: Vec<Option<usize>>,
  options: Vec<Vec<usize>>,
  // entropy: Vec<usize>,
  adjacency_rules: AdjacencyData,
  patterns: P,
  history: Vec<((usize, usize), HashSet<usize>, Vec<Vec<usize>>)>,
}

impl<P: Drawable + Clone> Grid<P> {
  pub fn new(width: usize, height: usize, adjacency_rules: &AdjacencyData, patterns: &P) -> Self {
    let patterns_length = patterns.len();
    Self {
      width,
      height,
      cells: vec![None; width * height],
      options: vec![(0..patterns_length).collect(); width * height],
      // entropy: vec![patterns_length; width * height],
      adjacency_rules: adjacency_rules.clone(),
      patterns: patterns.clone(),
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
        self.patterns.draw(x, y, *p);
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
          self.adjacency_rules[pattern].get(&(dx, dy)).unwrap().clone()
        } else {
          let vec1: Vec<usize> = self.options[idx].iter().flat_map(|opt| self.adjacency_rules[*opt].get(&(dx, dy)).unwrap().clone()).collect();
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
  // let image = load_texture("pat-tree.png").await.expect("image should be loaded").get_texture_data();
  // let processor = OverlappingPreprocessor::new(3, true, true, false);
  let image = load_texture("tiles-standard.png").await.expect("image should be loaded").get_texture_data();
  let processor = TileProcessor::new(32., true);
  let (patterns, adjacency_rules) = process(&processor, &image);
  let mut play = true;
  let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
  let seed = since_the_epoch.as_secs();
  srand(seed);

  let width = (SCREEN_WIDTH / TILE_SIZE) as usize;
  let height = (SCREEN_HEIGHT / TILE_SIZE) as usize;
  let mut grid = Grid::new(
    width,
    height,
    &adjacency_rules,
    &patterns
  );

  loop {
    clear_background(DARKGRAY);

    // tile_model::draw_patterns(&patterns, 200., "TILE PATTERNS:");
    if is_key_released(KeyCode::R) {
      grid = Grid::new(
        width,
        height,
        &adjacency_rules,
        &patterns
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
