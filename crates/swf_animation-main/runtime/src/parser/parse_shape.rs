use std::collections::HashMap;

use swf::{CharacterId, Shape};
use tessellator::{Mesh, ShapeTessellator};

use super::bitmap::CompressedBitmap;

pub mod matrix;
pub mod shape_utils;
pub mod tessellator;

pub struct Graphic {
    pub shape: Shape,
    pub lyon_mesh: Mesh,
}

pub fn parse_shape_and_bitmap(
    shapes: HashMap<CharacterId, Shape>,
    bitmaps: &HashMap<CharacterId, CompressedBitmap>,
) -> HashMap<CharacterId, Graphic> {
    let mut graphics = HashMap::new();
    let mut tessellator = ShapeTessellator::default();
    for (id, shape) in shapes {
        let distilled_shape = &shape;
        let lyon_mesh = tessellator.tessellate_shape(distilled_shape.into(), bitmaps);
        graphics.insert(id, Graphic { shape, lyon_mesh });
    }
    graphics
}
