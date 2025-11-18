use macroquad::prelude::*;

use crate::{assets::Assets, utils::*};

#[derive(Clone)]
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

pub struct LevelRenderer<'a> {
    pub assets: &'a Assets,
    pub camera: Camera2D,
    pub size: Vec2,
    pub clear_color: Color,
}
impl<'a> LevelRenderer<'a> {
    pub fn set_tile(&mut self, level: &mut Level, x: usize, y: usize, tile: [u8; 2]) {
        level.tiles[x + y * level.width] = tile;
        set_camera(&self.camera);
        draw_rectangle((x * 8) as f32, (y * 8) as f32, 8.0, 8.0, self.clear_color);

        for tile in tile {
            if tile == 0 {
                continue;
            }
            let tile = tile - 1;
            self.assets.tileset.draw_tile(
                (x * 8) as f32,
                (y * 8) as f32,
                (tile % 32) as f32,
                (tile / 32) as f32,
                None,
            );
        }
    }
    pub fn draw_level(level: &Level, assets: &Assets) {
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
    }
    pub fn new(level: &Level, assets: &'a Assets, clear_color: Color) -> Self {
        let mut camera = create_camera((level.width * 8) as f32, (level.height() * 8) as f32);
        camera.target = vec2(
            (level.width * 8) as f32 / 2.0,
            (level.height() * 8) as f32 / 2.0,
        );
        set_camera(&camera);
        clear_background(clear_color);

        Self::draw_level(level, assets);
        Self {
            assets,
            size: vec2((level.width * 8) as f32, (level.height() * 8) as f32),
            camera,
            clear_color,
        }
    }
}
