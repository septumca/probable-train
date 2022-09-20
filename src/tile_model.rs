use std::collections::{HashMap};

use macroquad::prelude::*;

use crate::{Drawable, WfcPreprocessor, TILE_SIZE, utils::{rotate_image, N_INDEXES}};

const OPPOSITE_EDGES_INDEXES: [(usize, usize); 4] = [(0, 2), (1, 3), (2, 0), (3, 1)];

#[derive(Clone)]
pub struct TexturePattern(Vec<Texture2D>);

impl Drawable for TexturePattern {
  fn draw(&self, x: f32, y: f32, idx: usize) {
    draw_texture_ex(
      self
      .0[idx], x, y,
      WHITE,
      DrawTextureParams {
        dest_size: Some(Vec2::splat(TILE_SIZE)),
        ..Default::default()
      }
    );
  }
  fn len(&self) -> usize {
    self.0.len()
  }
}

pub struct TileProcessor {
  rotate: bool,
  tile_size: f32,
}

impl TileProcessor {
  pub fn new(tile_size: f32, rotate: bool) -> Self {
    Self {
      rotate,
      tile_size,
    }
  }
}

impl WfcPreprocessor for TileProcessor {
  type Pattern = TexturePattern;

  fn create_patterns(&self, images: &[Image]) -> Self::Pattern {
    let patterns: Vec<Texture2D> = images.iter()
      .map(|i| {
        let tex = Texture2D::from_image(i);
        tex.set_filter(FilterMode::Linear);
        tex
      })
      .collect();
      TexturePattern(patterns)
  }

  fn create_adjacency_rules(&self, images: &[Image]) -> crate::utils::AdjacencyData {
    let edge_data = get_edges_for_images(images);
    let mut adjacenncy_data = vec![];

    for (idx, _) in images.iter().enumerate() {
      let source_edge_connections = edge_data[idx];
      let mut adjacencies : HashMap<(isize, isize), Vec<usize>> = HashMap::new();
      for (nx, ny) in N_INDEXES {
        adjacencies.insert((nx, ny), vec![]);
      }

      for (target_idx, _) in images.iter().enumerate() {
        let target_edge_connections = edge_data[target_idx];

        for (n_idx, (src_edge, target_edge)) in OPPOSITE_EDGES_INDEXES.iter().enumerate() {
          if source_edge_connections[*src_edge] == target_edge_connections[*target_edge] {
            let (nx, ny) = N_INDEXES[n_idx];
            adjacencies.get_mut(&(nx, ny)).unwrap().push(target_idx);
          }
        }
      }
      adjacenncy_data.push(adjacencies);
    }

    adjacenncy_data
  }

  fn extract_images(&self, image: &Image) -> Vec<Image> {
    let mut images: Vec<Image> = vec![];

    for x in (0..image.width).step_by(self.tile_size as usize) {
      for y in (0..image.height).step_by(self.tile_size as usize) {
        let img = image.sub_image(Rect::new(x as f32, y as f32, self.tile_size, self.tile_size));
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

fn color_to_slice(c: &Color) -> [u8; 4] {
  [(c.r * 255.) as u8, (c.g * 255.) as u8, (c.b * 255.) as u8, (c.a * 255.) as u8]
}

fn get_edge_colors(img: &Image) -> [Vec<Color>; 4] {

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

#[derive(Hash, PartialEq, Eq)]
struct VecU8(Vec<[u8; 4]>);

fn get_edges_for_images(images: &[Image]) -> Vec<[usize; 4]> {
  let mut idx = 0;
  let mut edges_store : HashMap<VecU8, usize> = HashMap::new();
  let mut edges: Vec<[usize; 4]> = vec![];

  for img in images {
    let mut edge_indexes = vec![None; 4];

    for (side_idx, e) in get_edge_colors(img).iter().enumerate() {
      let c_bytes = VecU8(e.into_iter().map(|c| color_to_slice(c)).collect());
      if let Some(i) = edges_store.get(&c_bytes) {
        edge_indexes[side_idx] = Some(i.clone());
      } else {
        edge_indexes[side_idx] = Some(idx);
        edges_store.insert(c_bytes, idx);
        idx += 1;
      }
    }

    edges.push([edge_indexes[0].unwrap(), edge_indexes[1].unwrap(), edge_indexes[2].unwrap(), edge_indexes[3].unwrap()]);
  }

  edges
}
