use macroquad::prelude::*;
use std::{rc::Rc, cell::RefCell};

const TILE_NAME: String = "tiles.png".to_owned();
const TILE_SIZE: f32 = 32.;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EdgePartType {
  Path,
  Forest,
  Coast,
  Water,
}

const EDGES: [[[EdgePartType; 3]; 4]; 9] = [
  [
    [
      EdgePartType::Forest,
      EdgePartType::Coast,
      EdgePartType::Water,
    ],
    [
      EdgePartType::Water,
      EdgePartType::Coast,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
  ],
  [
    [
      EdgePartType::Forest,
      EdgePartType::Coast,
      EdgePartType::Water,
    ],
    [
      EdgePartType::Water,
      EdgePartType::Water,
      EdgePartType::Water,
    ],
    [
      EdgePartType::Water,
      EdgePartType::Coast,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
  ], //
  [
    [
      EdgePartType::Water,
      EdgePartType::Water,
      EdgePartType::Water,
    ],
    [
      EdgePartType::Water,
      EdgePartType::Water,
      EdgePartType::Water,
    ],
    [
      EdgePartType::Water,
      EdgePartType::Water,
      EdgePartType::Water,
    ],
    [
      EdgePartType::Water,
      EdgePartType::Water,
      EdgePartType::Water,
    ],
  ],//
  [
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
  ], //
  [
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
  ],//
  [
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
  ],
  [
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
  ],
  [
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Path,
      EdgePartType::Forest,
    ],
  ],
  [
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
    [
      EdgePartType::Forest,
      EdgePartType::Forest,
      EdgePartType::Forest,
    ],
  ],
];

type TileResource = (Rc<RefCell<Texture2D>>,[[EdgePartType; 3]; 4]);

struct Resources {
  sea: TileResource,
  coast: TileResource,
  coast_curve: TileResource,
  x_section: TileResource,
  t_section: TileResource,
  line: TileResource,
  curve: TileResource,
  dead_end: TileResource,
  blank: TileResource,
}

impl Resources {
  pub fn new(image: Image) -> Self {

    let data = vec![];
    for i in 0..9 {
      let mut tex = Texture2D::from_image(&image.sub_image(Rect::new(TILE_SIZE * i as f32, 0., TILE_SIZE, TILE_SIZE)));
      tex.set_filter(FilterMode::Nearest);
      data.push((Rc::new(RefCell::new(tex)), EDGES[i]))
    }

    let resources = Resources {
      coast_curve: data[0],
      coast: data[1],
      sea: data[2],
      dead_end: data[3],
      line: data[4],
      curve: data[5],
      x_section: data[6],
      t_section: data[7],
      blank: data[8],
    };

    resources
  }
}