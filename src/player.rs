use macroquad::prelude::*;

use crate::{assets::Assets, level::Level, utils::*};

pub struct Player {
    pub pos: Vec2,
    pub camera_pos: Vec2,
    pub velocity: Vec2,
    pub move_vector: Vec2,
    pub crouching: bool,

    pub time: f32,

    pub grounded: bool,
    pub jump_frames: f32,

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
            grounded: true,
            moving: false,
            crouching: false,
            jump_frames: 0.0,
        }
    }
    pub fn update(&mut self, delta_time: f32, level: &Level) {
        self.velocity.y += GRAVITY * delta_time;
        self.time += delta_time;
        let mut forces = Vec2::ZERO;

        let mut friction_mod = 1.0;

        let input = get_input_axis();
        self.moving = input.x != 0.0;
        if self.moving {
            self.move_vector = input;
        } else {
            friction_mod = 5.0;
        }

        if self.velocity.x.abs() > input.x.abs()
            && ((self.velocity.x < 0.0 && input.x > 0.0)
                || (self.velocity.x > 0.0 && input.x < 0.0))
        {
            self.velocity.x = 0.0;
        } else {
        }
        forces.x += input.x * ACCELERATION;
        forces.x -= self.velocity.x
            * if self.grounded {
                GROUND_FRICTION * friction_mod
            } else {
                AIR_DRAG * friction_mod
            };

        if self.grounded {
            self.jump_frames = 0.0;
        }
        if is_key_down(KeyCode::Space)
            && (self.grounded || (self.jump_frames > 0.0 && self.jump_frames < 0.5))
        {
            if self.jump_frames == 0.0 {
                self.velocity.y -= 3.6 * 60.0;
            } else {
                self.velocity.y -= 65.0 * 10.0 * delta_time;
                //forces.y -= 60.0 * 10.0;
            }
            self.jump_frames += delta_time;
        }

        self.velocity += forces * delta_time;
        self.velocity.x = self.velocity.x.clamp(-MAX_VELOCITY, MAX_VELOCITY);

        (self.pos, self.grounded) = update_physicsbody(
            self.pos,
            &mut self.velocity,
            delta_time,
            level,
            !self.crouching,
        );
        self.camera_pos = self.pos;
    }

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
            self.pos.floor().x - 4.0,
            self.pos.floor().y - 8.0,
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
            self.pos.floor().x - 4.0,
            self.pos.floor().y - 8.0,
            WHITE,
            DrawTextureParams {
                flip_x: self.move_vector.x < 0.0,
                ..Default::default()
            },
        );

        // draw speedometer
        if DEBUG_ARGS.speedometer {
            let amt = self.velocity.x.abs() / MAX_VELOCITY;
            let width = 100.0;
            let height = 16.0;
            let x = 16.0 + self.camera_pos.x.floor() - SCREEN_WIDTH / 2.0;
            let y = 16.0 + self.camera_pos.y.floor() - SCREEN_HEIGHT / 2.0;
            draw_rectangle(x - 1.0, y - 1.0, width + 2.0, height + 2.0, BLACK);
            draw_rectangle(x, y, width * amt, height, LIME);
        }
    }
}

fn ceil_g(a: f32) -> f32 {
    if a < 0.0 { a.floor() } else { a.ceil() }
}
pub fn update_physicsbody(
    pos: Vec2,
    velocity: &mut Vec2,
    delta_time: f32,
    world: &Level,
    tall: bool,
) -> (Vec2, bool) {
    let mut grounded = false;
    let mut new = pos + *velocity * delta_time;

    let tile_x = pos.x / 8.0;
    let tile_y = pos.y / 8.0;

    let mut tiles_y = vec![
        (tile_x.trunc(), ceil_g(new.y / 8.0)),
        (ceil_g(tile_x), ceil_g(new.y / 8.0)),
        (tile_x.trunc(), (new.y / 8.0).trunc()),
        (ceil_g(tile_x), (new.y / 8.0).trunc()),
    ];
    if tall {
        tiles_y.push((tile_x.trunc(), (new.y / 8.0).trunc() - 1.0));
        tiles_y.push((ceil_g(tile_x), (new.y / 8.0).trunc() - 1.0));
    }

    for (tx, ty) in tiles_y {
        let tile = world.get_tile((tx / 2.0) as usize, (ty / 2.0) as usize)[0];
        if tile != 0 {
            let c = if velocity.y < 0.0 {
                tile_y.floor() * 8.0
            } else {
                grounded = true;
                tile_y.ceil() * 8.0
            };
            new.y = c;
            velocity.y = 0.0;
            break;
        }
    }
    let mut tiles_x = vec![
        ((new.x / 8.0).trunc(), ceil_g(new.y / 8.0)),
        (ceil_g(new.x / 8.0), ceil_g(new.y / 8.0)),
        (ceil_g(new.x / 8.0), (new.y / 8.0).trunc()),
        ((new.x / 8.0).trunc(), (new.y / 8.0).trunc()),
    ];
    if tall {
        tiles_x.push(((new.x / 8.0).trunc(), (new.y / 8.0).trunc() - 1.0));
        tiles_x.push((ceil_g(new.x / 8.0), (new.y / 8.0).trunc() - 1.0));
    }

    for (tx, ty) in tiles_x {
        let tile = world.get_tile((tx / 2.0) as usize, (ty / 2.0) as usize)[0];
        if tile != 0 {
            let c = if velocity.x < 0.0 {
                tile_x.floor() * 8.0
            } else {
                tile_x.ceil() * 8.0
            };
            new.x = c;
            velocity.x = 0.0;
            break;
        }
    }
    (new, grounded)
}
