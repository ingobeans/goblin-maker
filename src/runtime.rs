use crate::{
    assets::Assets,
    level::{Level, LevelRenderer},
    player::Player,
    utils::*,
};
use macroquad::{miniquad::window::screen_size, prelude::*};

pub struct GoblinRuntime<'a> {
    assets: &'a Assets,
    player: Player,
    level: Level,
    level_renderer: LevelRenderer<'a>,
    pixel_camera: Camera2D,
}

impl<'a> GoblinRuntime<'a> {
    pub fn new(assets: &'a Assets, level: Level) -> Self {
        Self {
            level_renderer: LevelRenderer::new(&level, assets, BLACK.with_alpha(0.0)),
            assets,
            player: Player::new(level.player_spawn),
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
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_x = mouse_x / scale_factor;
        let mouse_y = mouse_y / scale_factor;

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

        draw_text(&get_fps().to_string(), 64.0, 64.0, 32.0, WHITE);
    }
}
