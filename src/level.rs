use macroquad::prelude::*;

use crate::{assets::Assets, utils::*};

#[derive(Clone)]
pub enum Character {}

#[derive(Clone)]
pub struct Level {
    pub tiles: Vec<[u8; 2]>,
    pub width: usize,
    pub player_spawn: Vec2,
    pub characters: Vec<(Vec2, Character)>,
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
        draw_rectangle(
            (x * 16) as f32,
            (y * 16) as f32,
            16.0,
            16.0,
            self.clear_color,
        );

        for (tile, spritesheet) in tile.into_iter().zip(
            [
                &self.assets.terrain_tileset,
                &self.assets.decoration_tileset,
            ]
            .iter(),
        ) {
            if tile == 0 {
                continue;
            }
            let tile = tile - 1;
            spritesheet.draw_tile(
                (x * 16) as f32,
                (y * 16) as f32,
                (tile % 3) as f32,
                (tile / 3) as f32,
                None,
            );
        }
    }
    pub fn draw_level(level: &Level, assets: &Assets) {
        for (index, tile_bundle) in level.tiles.iter().enumerate() {
            for (tile, tileset) in tile_bundle
                .into_iter()
                .zip([&assets.terrain_tileset, &assets.decoration_tileset].iter())
            {
                if *tile == 0 {
                    continue;
                }
                let tile = tile - 1;
                tileset.draw_tile(
                    ((index % level.width) * 16) as f32,
                    ((index / level.width) * 16) as f32,
                    (tile % 3) as f32,
                    (tile / 3) as f32,
                    None,
                );
            }
        }
    }
    pub fn new(level: &Level, assets: &'a Assets, clear_color: Color) -> Self {
        let mut camera = create_camera((level.width * 16) as f32, (level.height() * 16) as f32);
        camera.target = vec2(
            (level.width * 16) as f32 / 2.0,
            (level.height() * 16) as f32 / 2.0,
        );
        set_camera(&camera);
        clear_background(clear_color);

        Self::draw_level(level, assets);
        Self {
            assets,
            size: vec2((level.width * 16) as f32, (level.height() * 16) as f32),
            camera,
            clear_color,
        }
    }
}
