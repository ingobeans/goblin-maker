use macroquad::prelude::*;
use nanoserde::{DeBin, SerBin};

use crate::{
    assets::{Assets, Spritesheet},
    utils::*,
};

#[derive(Clone, DeBin, SerBin)]
pub enum Character {
    PlayerSpawn,
    Checkpoint,
    WanderEnemy(usize),
}

#[derive(Clone, DeBin, SerBin)]
pub struct Level {
    pub tiles: Vec<[u8; 2]>,
    pub width: usize,
    pub characters: Vec<((f32, f32), Character, usize)>,
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
    fn draw_tile(
        level: &Level,
        level_pos: (usize, usize),
        screen: Vec2,
        tile: Vec2,
        spritesheet: &Spritesheet,
    ) {
        if tile == Vec2::ZERO
            && let Some((autotile_hashmap, autotile_tileset)) = &spritesheet.autotile_first
        {
            let sides = [
                level.get_tile(level_pos.0, level_pos.1.saturating_sub(1))[0] == 1,
                level.get_tile(level_pos.0.saturating_sub(1), level_pos.1)[0] == 1,
                level.get_tile(level_pos.0 + 1, level_pos.1)[0] == 1,
                level.get_tile(level_pos.0, level_pos.1 + 1)[0] == 1,
            ];
            let pos = autotile_hashmap.get(&sides).unwrap();
            autotile_tileset.draw_tile(screen.x, screen.y, pos.x / 16.0, pos.y / 16.0, None);
            return;
        }

        spritesheet.draw_tile(screen.x, screen.y, tile.x, tile.y, None);
    }
    pub fn set_tile(&mut self, level: &mut Level, x: usize, y: usize, tile_bundle: [u8; 2]) {
        level.tiles[x + y * level.width] = tile_bundle;
        set_camera(&self.camera);
        draw_rectangle(
            (x * 16) as f32,
            (y * 16) as f32,
            16.0,
            16.0,
            self.clear_color,
        );

        for (index, (tile, spritesheet)) in tile_bundle
            .into_iter()
            .zip(
                [
                    &self.assets.terrain_tileset,
                    &self.assets.decoration_tileset,
                ]
                .iter(),
            )
            .enumerate()
        {
            let tile_positions = [
                (x, y.saturating_sub(1)),
                (x.saturating_sub(1), y),
                (x + 1, y),
                (x, y + 1),
            ];
            if index == 0 {
                for (x, y) in tile_positions {
                    if y >= level.height() {
                        continue;
                    }
                    if x >= level.width {
                        continue;
                    }
                    let tile = level.tiles[x + y * level.width];
                    if tile[0] == 0 {
                        continue;
                    }
                    Self::draw_tile(
                        level,
                        (x, y),
                        vec2((x * 16) as f32, (y * 16) as f32),
                        vec2(((tile[0] - 1) % 3) as f32, ((tile[0] - 1) / 3) as f32),
                        &self.assets.terrain_tileset,
                    );
                    if tile[1] != 0 {
                        Self::draw_tile(
                            level,
                            (x, y),
                            vec2((x * 16) as f32, (y * 16) as f32),
                            vec2(((tile[1] - 1) % 3) as f32, ((tile[1] - 1) / 3) as f32),
                            &self.assets.decoration_tileset,
                        );
                    }
                }
            }
            if tile == 0 {
                continue;
            }
            let tile = tile - 1;

            Self::draw_tile(
                level,
                (x, y),
                vec2((x * 16) as f32, (y * 16) as f32),
                vec2((tile % 3) as f32, (tile / 3) as f32),
                spritesheet,
            );
        }
    }
    pub fn draw_level(level: &Level, assets: &Assets) {
        for (index, tile_bundle) in level.tiles.iter().enumerate() {
            for (tile, tileset) in tile_bundle
                .iter()
                .zip([&assets.terrain_tileset, &assets.decoration_tileset].iter())
            {
                if *tile == 0 {
                    continue;
                }
                let tile = tile - 1;
                Self::draw_tile(
                    level,
                    ((index % level.width), (index / level.width)),
                    vec2(
                        ((index % level.width) * 16) as f32,
                        ((index / level.width) * 16) as f32,
                    ),
                    vec2((tile % 3) as f32, (tile / 3) as f32),
                    tileset,
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
