use macroquad::prelude::*;

use crate::{assets::Assets, level::Level};

pub struct Player {
    pub pos: Vec2,
    pub camera_pos: Vec2,
    pub velocity: Vec2,
    pub move_vector: Vec2,

    pub time: f32,

    pub moving: bool,
}
impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            camera_pos: pos,
            velocity: Vec2::ZERO,
            move_vector: Vec2::ZERO,
            time: 0.0,
            moving: false,
        }
    }
    pub fn update(&mut self, delta_time: f32) {}

    pub fn draw(&mut self, assets: &Assets) {
        let legs_animation = if self.moving && self.velocity.length() > 1.0 {
            "walk"
        } else {
            "idle"
        };
        let torso_animation = if self.velocity.y < -1.0 {
            "jump"
        } else {
            "idle"
        };

        draw_texture_ex(
            assets
                .player_legs
                .get_by_name(legs_animation)
                .get_at_time((self.time * 1000.0) as u32),
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                flip_x: self.move_vector.x < 0.0,
                ..Default::default()
            },
        );
        draw_texture_ex(
            assets
                .player_torso
                .get_by_name(torso_animation)
                .get_at_time((self.time * 1000.0) as u32),
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                flip_x: self.move_vector.x < 0.0,
                ..Default::default()
            },
        );
    }
}

fn ceil_g(a: f32) -> f32 {
    if a < 0.0 { a.floor() } else { a.ceil() }
}
pub fn update_physicsbody(pos: Vec2, velocity: &mut Vec2, delta_time: f32, world: &Level) -> Vec2 {
    let mut new = pos + *velocity * delta_time;

    let tile_x = pos.x / 16.0;
    let tile_y = pos.y / 16.0;

    let tiles_y = [
        (tile_x.trunc(), ceil_g(new.y / 16.0)),
        (ceil_g(tile_x), ceil_g(new.y / 16.0)),
        (tile_x.trunc(), (new.y / 16.0).trunc()),
        (ceil_g(tile_x), (new.y / 16.0).trunc()),
    ];

    for (tx, ty) in tiles_y {
        let tile = world.get_tile(tx as usize, ty as usize)[0];
        if tile != 0 {
            let c = if velocity.y < 0.0 {
                tile_y.floor() * 16.0
            } else {
                tile_y.ceil() * 16.0
            };
            new.y = c;
            velocity.y = 0.0;
            break;
        }
    }
    let tiles_x = [
        ((new.x / 16.0).trunc(), ceil_g(new.y / 16.0)),
        (ceil_g(new.x / 16.0), ceil_g(new.y / 16.0)),
        (ceil_g(new.x / 16.0), (new.y / 16.0).trunc()),
        ((new.x / 16.0).trunc(), (new.y / 16.0).trunc()),
    ];

    for (tx, ty) in tiles_x {
        let tile = world.get_tile(tx as usize, ty as usize)[0];
        if tile != 0 {
            let c = if velocity.x < 0.0 {
                tile_x.floor() * 16.0
            } else {
                tile_x.ceil() * 16.0
            };
            new.x = c;
            velocity.x = 0.0;
            break;
        }
    }
    new
}
