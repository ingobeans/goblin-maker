use crate::{
    assets::{Animation, Assets},
    level::{Character, Level, LevelRenderer},
    player::{Player, PlayerUpdateResult, update_physicsbody},
    ui::*,
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
    pub death_frames: f32,
}
#[derive(Clone, Copy)]
enum RuntimeMenu {
    None,
    Paused,
    Win,
}
impl RuntimeMenu {
    fn toggle(&mut self) {
        match self {
            RuntimeMenu::Paused => *self = RuntimeMenu::None,
            RuntimeMenu::None => *self = RuntimeMenu::Paused,
            RuntimeMenu::Win => {}
        }
    }
    fn get_title(&self) -> &'static str {
        match self {
            RuntimeMenu::Paused => "Paused",
            RuntimeMenu::Win => "Level Complete",
            RuntimeMenu::None => panic!(),
        }
    }
}
pub enum RuntimeResult {
    None,
    Win,
    Exit,
}
pub struct GoblinRuntime<'a> {
    assets: &'a Assets,
    player: Player,
    level: Level,
    level_renderer: LevelRenderer<'a>,
    pixel_camera: Camera2D,
    enemies: Vec<AliveEnemy<'a>>,
    menu: RuntimeMenu,
    pub level_details: Option<(String, String)>,
}

impl<'a> GoblinRuntime<'a> {
    pub fn new(assets: &'a Assets, level: Level, level_name: Option<(String, String)>) -> Self {
        Self {
            enemies: level
                .characters
                .iter()
                .filter_map(|(pos, character, _)| match character {
                    Character::PlayerSpawn => None,
                    Character::Flag => None,
                    Character::WanderEnemy(animation) => Some(AliveEnemy::new(
                        vec2(pos.0, pos.1) + vec2(0.0, 8.0),
                        &assets.enemies.animations[*animation],
                        0.0,
                        true,
                        Vec2::ZERO,
                        0.0,
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
            menu: RuntimeMenu::None,
            level_details: level_name,
        }
    }
    pub fn update(&mut self) -> RuntimeResult {
        // cap delta time to a minimum of 60 fps.
        let delta_time = get_frame_time().min(1.0 / 60.0);
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);

        let result = self.player.update(delta_time, &self.level);
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
        self.enemies.retain_mut(|enemy| {
            if enemy.death_frames > 0.0 {
                enemy.death_frames += delta_time;
                draw_texture_ex(
                    enemy.animation.get_at_time((enemy.time * 1000.0) as u32),
                    enemy.pos.x - 12.0,
                    enemy.pos.y - 4.0,
                    WHITE,
                    DrawTextureParams {
                        flip_x: !enemy.moving_left,
                        dest_size: Some(vec2(32.0, 18.0)),
                        ..Default::default()
                    },
                );
            } else {
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
                if !self.player.died
                    && self.player.victory == 0.0
                    && self.player.pos.distance_squared(enemy.pos) < 200.0
                {
                    if self.player.pos.y < enemy.pos.y {
                        enemy.death_frames += delta_time;
                        self.player.velocity.y = -3.6 * 60.0;
                    } else {
                        self.player.die();
                    }
                }
            }
            enemy.death_frames < 0.5
        });
        if self.player.victory == 0.0
            && !self.player.died
            && (self.player.pos + vec2(-4.0, 0.0))
                .distance_squared(self.level.characters[1].0.into())
                < 140.0
        {
            self.player.victory = delta_time;
            self.menu = RuntimeMenu::Win;
        }
        self.player.draw(self.assets);

        self.assets.character_tileset.draw_tile(
            self.level.characters[1].0.0,
            self.level.characters[1].0.1,
            1.0,
            0.0,
            None,
        );
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
        if !matches!(self.menu, RuntimeMenu::None) {
            let size = vec2(150.0, 150.0);
            let mut pos = ((vec2(actual_screen_width, actual_screen_height) - size * scale_factor)
                / 2.0)
                .floor();

            if matches!(self.menu, RuntimeMenu::Win) {
                let amt = (self.player.victory / 0.6).min(1.0);
                let amt = 2.0 * 1.5_f32.powf(amt.powi(2)) - 2.0;
                pos.y = (-size.y * scale_factor).lerp(pos.y, amt);
            }

            let rect = UIRect::new(
                pos,
                size * scale_factor,
                MAKER_BG_COLOR,
                (scale_factor, BLACK),
            );
            rect.draw();
            let font_size = (20.0 * scale_factor) as u16;
            draw_text_ex(
                self.menu.get_title(),
                pos.x + 5.0 * scale_factor,
                pos.y + font_size as f32,
                TextParams {
                    font_size,
                    font: Some(&self.assets.font),
                    ..Default::default()
                },
            );
            let font_size = (12.0 * scale_factor) as u16;
            if let Some((name, author)) = &self.level_details {
                let font_size = (8.0 * scale_factor) as u16;
                draw_multiline_text_ex(
                    &format!("Level name: {name}\nLevel author: {author}"),
                    pos.x + 5.0 * scale_factor,
                    pos.y + font_size as f32 + 30.0 * scale_factor,
                    None,
                    TextParams {
                        color: LIGHTGRAY,
                        font_size,
                        font: Some(&self.assets.font),
                        ..Default::default()
                    },
                );
            } else if matches!(self.menu, RuntimeMenu::Win) {
                let font_size = (8.0 * scale_factor) as u16;
                draw_multiline_text_ex(
                    "Level is now verified until further\nchanges and can be uploaded!",
                    pos.x + 5.0 * scale_factor,
                    pos.y + font_size as f32 + 30.0 * scale_factor,
                    None,
                    TextParams {
                        color: LIGHTGRAY,
                        font_size,
                        font: Some(&self.assets.font),
                        ..Default::default()
                    },
                );
            }

            let btn_size = vec2(135.0, 20.0);
            if matches!(self.menu, RuntimeMenu::Paused) {
                let resume = UITextButton::new(
                    pos + vec2((size.x - btn_size.x) / 2.0, size.y - 2.0 * btn_size.y - 7.0)
                        * scale_factor,
                    btn_size * scale_factor,
                    "Resume".to_string(),
                    SKY_COLOR,
                    MAKER_BG_COLOR,
                    (scale_factor, BLACK),
                    (font_size, &self.assets.font, 5.0 * scale_factor),
                );
                if resume.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
                    self.menu = RuntimeMenu::None;
                }
                resume.draw();
            }
            let return_text = if self.level_details.is_some() {
                "Return to menu"
            } else {
                "Return to editor"
            };
            let return_to_menu = UITextButton::new(
                pos + vec2((size.x - btn_size.x) / 2.0, size.y - btn_size.y - 5.0) * scale_factor,
                btn_size * scale_factor,
                return_text.to_string(),
                SKY_COLOR,
                MAKER_BG_COLOR,
                (scale_factor, BLACK),
                (font_size, &self.assets.font, 5.0 * scale_factor),
            );
            return_to_menu.draw();
            if return_to_menu.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
                if matches!(self.menu, RuntimeMenu::Win) {
                    return RuntimeResult::Win;
                }
                return RuntimeResult::Exit;
            }
        } else {
            let pause_btn = UIImageButton::new(
                vec2(2.0, 2.0) * scale_factor,
                &self.assets.pause_btn.frames[0].0,
                &self.assets.pause_btn.frames[1].0,
                scale_factor,
                false,
            );
            if pause_btn.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
                self.menu = RuntimeMenu::Paused;
            }
            pause_btn.draw();
        }
        if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Escape) {
            self.menu.toggle();
        }
        if let PlayerUpdateResult::GameOver = result {
            let mut level = Level {
                tiles: Vec::new(),
                width: 0,
                characters: Vec::new(),
            };
            std::mem::swap(&mut level, &mut self.level);
            let mut new = GoblinRuntime::new(self.assets, level, self.level_details.clone());
            new.menu = self.menu;
            *self = new;
        }
        RuntimeResult::None
    }
}
