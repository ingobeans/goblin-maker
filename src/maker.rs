use crate::{
    assets::Assets,
    level::{Level, LevelRenderer},
    player::Player,
    utils::*,
};
use macroquad::{miniquad::window::screen_size, prelude::*};

pub struct GoblinMaker<'a> {
    assets: &'a Assets,
    pub level: Level,
    level_renderer: LevelRenderer<'a>,
    camera_pos: Vec2,
    camera_zoom: f32,
}

impl<'a> GoblinMaker<'a> {
    pub fn new(assets: &'a Assets) -> Self {
        let width = 100;
        let height = 50;
        let player_spawn = vec2((width * 8 - 4) as f32, (height * 8 - 8) as f32);
        let level = Level {
            tiles: vec![[0, 0]; width * height],
            width,
            player_spawn,
        };
        let level_renderer = LevelRenderer::new(&level, assets, SKY_COLOR);

        // all these calculations are literally just to zoom out the viewport around the center of the level
        let screen = vec2(SCREEN_WIDTH, SCREEN_HEIGHT);
        let mut camera_zoom: f32 = 1.0;
        let mut camera_pos = level_renderer.size / 2.0 - screen / 2.0;
        let (mouse_x, mouse_y) = (screen.x / 2.0, screen.y / 2.0);
        let old_mouse_world_x = mouse_x + camera_pos.x - SCREEN_WIDTH / 2.0;
        let old_mouse_world_y = mouse_y + camera_pos.y - SCREEN_HEIGHT / 2.0;
        camera_zoom /= 2.0;
        camera_zoom = camera_zoom.max(MIN_ZOOM);
        camera_pos.x = old_mouse_world_x + SCREEN_WIDTH / 2.0 - mouse_x / camera_zoom;
        camera_pos.y = old_mouse_world_y + SCREEN_HEIGHT / 2.0 - mouse_y / camera_zoom;

        Self {
            level_renderer,
            assets,
            level,
            camera_zoom,
            camera_pos,
        }
    }
    pub fn update(&mut self) {
        let delta_time = get_frame_time();
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_x = mouse_x / scale_factor;
        let mouse_y = mouse_y / scale_factor;
        let (mouse_tile_x, mouse_tile_y) = (
            (((mouse_x) / self.camera_zoom + self.camera_pos.x) / 16.0).floor(),
            (((mouse_y) / self.camera_zoom + self.camera_pos.y) / 16.0).floor(),
        );
        let cursor_tile = if mouse_tile_x >= 0.0
            && mouse_tile_y >= 0.0
            && mouse_tile_x < self.level.width as f32
            && mouse_tile_y < self.level.height() as f32
        {
            Some((mouse_tile_x as usize, mouse_tile_y as usize))
        } else {
            None
        };

        let mouse_delta = mouse_delta_position();
        let scroll = mouse_wheel();

        // handle panning with middle-mouse
        if is_mouse_button_down(MouseButton::Middle) {
            self.camera_pos.x +=
                mouse_delta.x as f32 * actual_screen_width / scale_factor / 2.0 / self.camera_zoom;
            self.camera_pos.y +=
                mouse_delta.y as f32 * actual_screen_height / scale_factor / 2.0 / self.camera_zoom;
        }
        // handle scrolling
        if scroll.1 != 0.0 {
            let amt = if scroll.1 > 0.0 {
                1.0 / SCROLL_AMT
            } else {
                SCROLL_AMT
            };
            // store old mouse position (in world position)
            let old_mouse_world_x =
                mouse_x / self.camera_zoom + self.camera_pos.x - SCREEN_WIDTH / 2.0;
            let old_mouse_world_y =
                mouse_y / self.camera_zoom + self.camera_pos.y - SCREEN_HEIGHT / 2.0;

            // update grid size
            self.camera_zoom /= amt;
            self.camera_zoom = self.camera_zoom.max(MIN_ZOOM);
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            self.camera_pos.x = old_mouse_world_x + SCREEN_WIDTH / 2.0 - mouse_x / self.camera_zoom;
            self.camera_pos.y =
                old_mouse_world_y + SCREEN_HEIGHT / 2.0 - mouse_y / self.camera_zoom;
        }

        // handle clicking
        if is_mouse_button_down(MouseButton::Left)
            && let Some((tx, ty)) = cursor_tile
        {
            self.level_renderer
                .set_tile(&mut self.level, tx, ty, [1, 0]);
        }

        set_default_camera();
        clear_background(MAKER_BG_COLOR);
        draw_texture_ex(
            &self
                .level_renderer
                .camera
                .render_target
                .as_ref()
                .unwrap()
                .texture,
            -self.camera_pos.x * scale_factor * self.camera_zoom,
            -self.camera_pos.y * scale_factor * self.camera_zoom,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    self.level_renderer.size.x * scale_factor * self.camera_zoom,
                    self.level_renderer.size.y * scale_factor * self.camera_zoom,
                )),
                ..Default::default()
            },
        );
        let player_pos = vec2(
            (self.level.player_spawn.x - 4.0) * scale_factor * self.camera_zoom
                - self.camera_pos.x * scale_factor * self.camera_zoom,
            (self.level.player_spawn.y - 8.0) * scale_factor * self.camera_zoom
                - self.camera_pos.y * scale_factor * self.camera_zoom,
        );
        let params = DrawTextureParams {
            dest_size: Some(vec2(
                16.0 * scale_factor * self.camera_zoom,
                16.0 * scale_factor * self.camera_zoom,
            )),
            ..Default::default()
        };
        draw_texture_ex(
            self.assets.player_legs.animations[0].get_at_time(0),
            player_pos.x,
            player_pos.y,
            WHITE,
            params.clone(),
        );
        draw_texture_ex(
            self.assets.player_torso.animations[0].get_at_time(0),
            player_pos.x,
            player_pos.y,
            WHITE,
            params,
        );
    }
}
