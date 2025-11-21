use crate::{
    assets::{Animation, Assets},
    level::{Character, Level, LevelRenderer},
    player::{Player, update_physicsbody},
    utils::*,
};
use impl_new_derive::ImplNew;
use macroquad::{miniquad::window::screen_size, prelude::*};

#[derive(ImplNew)]
struct AliveEnemy<'a> {
    pub pos: Vec2,
    pub animation: &'a Animation,
    pub time: f32,
    pub moving_left: bool,
    pub velocity: Vec2,
}

pub struct GoblinRuntime<'a> {
    assets: &'a Assets,
    player: Player,
    level: Level,
    level_renderer: LevelRenderer<'a>,
    pixel_camera: Camera2D,
    enemies: Vec<AliveEnemy<'a>>,
}

impl<'a> GoblinRuntime<'a> {
    pub fn new(assets: &'a Assets, level: Level) -> Self {
        Self {
            enemies: level
                .characters
                .iter()
                .filter_map(|(pos, character, _)| match character {
                    Character::PlayerSpawn => None,
                    Character::Checkpoint => None,
                    Character::WanderEnemy(animation) => Some(AliveEnemy::new(
                        vec2(pos.0, pos.1) + vec2(0.0, 8.0),
                        &assets.enemies.animations[*animation],
                        0.0,
                        true,
                        Vec2::ZERO,
                    )),
                })
                .collect(),
            level_renderer: LevelRenderer::new(&level, assets, BLACK.with_alpha(0.0)),
            assets,
            player: Player::new(
                vec2(level.characters[0].0.0, level.characters[0].0.1) + vec2(4.0, 8.0),
            ),
            level,
            pixel_camera: create_camera(SCREEN_WIDTH, SCREEN_HEIGHT),
        }
    }
    pub fn update(&mut self) {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);

        self.player.update(delta_time, &self.level);
        self.pixel_camera.target = self.player.camera_pos.floor();
        set_camera(&self.pixel_camera);
        clear_background(SKY_COLOR);

        draw_texture_ex(
            &self
                .level_renderer
                .camera
                .render_target
                .as_ref()
                .unwrap()
                .texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams::default(),
        );
        self.player.draw(self.assets);
        for enemy in self.enemies.iter_mut() {
            enemy.time += delta_time;
            enemy.velocity.y += GRAVITY * delta_time;
            enemy.velocity.x = if enemy.moving_left { -1.0 } else { 1.0 } * 32.0;
            let old = enemy.velocity;
            enemy.pos = update_physicsbody(
                enemy.pos,
                &mut enemy.velocity,
                delta_time,
                &self.level,
                true,
            )
            .0;
            if old.x.abs() > enemy.velocity.x.abs() {
                enemy.moving_left = !enemy.moving_left;
            }

            draw_texture_ex(
                enemy.animation.get_at_time((enemy.time * 1000.0) as u32),
                enemy.pos.x - 12.0,
                enemy.pos.y - 16.0,
                WHITE,
                DrawTextureParams {
                    flip_x: !enemy.moving_left,
                    ..Default::default()
                },
            );
        }
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &self.pixel_camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    SCREEN_WIDTH * scale_factor,
                    SCREEN_HEIGHT * scale_factor,
                )),
                ..Default::default()
            },
        );
    }
}
