use macroquad::prelude::*;

use crate::{assets::Assets, utils::*};

pub struct Level {
    pub tiles: Vec<[u8; 2]>,
    pub width: usize,
}
impl Level {
    pub fn height(&self) -> usize {
        self.tiles.len() / self.width
    }
    pub fn get_tile(&self, x: usize, y: usize) -> [u8; 2] {
        if x >= self.width || y >= self.height() {
            return [0, 0];
        }
        self.tiles[x + y * self.width]
    }
}

pub struct LevelRenderer {
    pub camera: Camera2D,
}
impl LevelRenderer {
    pub fn new(level: &Level, assets: &Assets) -> Self {
        let mut camera = create_camera((level.width * 8) as f32, (level.height() * 8) as f32);
        camera.target = vec2(
            (level.width * 8) as f32 / 2.0,
            (level.height() * 8) as f32 / 2.0,
        );
        set_camera(&camera);
        clear_background(BLACK.with_alpha(0.0));

        for (index, tile_bundle) in level.tiles.iter().enumerate() {
            for tile in tile_bundle {
                if *tile == 0 {
                    continue;
                }
                let tile = tile - 1;
                assets.tileset.draw_tile(
                    ((index % level.width) * 8) as f32,
                    ((index / level.width) * 8) as f32,
                    (tile % 32) as f32,
                    (tile / 32) as f32,
                    None,
                );
            }
        }
        Self { camera }
    }
}
