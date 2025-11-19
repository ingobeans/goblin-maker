use crate::{
    assets::Assets,
    level::{Level, LevelRenderer},
    ui::*,
    utils::*,
};
use macroquad::{miniquad::window::screen_size, prelude::*};

enum Dragging {
    No,
    UiOwned,
    WorldOwned,
}

pub struct GoblinMaker<'a> {
    assets: &'a Assets,
    pub level: Level,
    level_renderer: LevelRenderer<'a>,
    camera_pos: Vec2,
    camera_zoom: f32,
    sidebar: (f32, u8, f32),
    dragging: Dragging,
}

impl<'a> GoblinMaker<'a> {
    pub fn new(assets: &'a Assets) -> Self {
        let width = 100;
        let height = 50;
        let player_spawn = vec2((width * 8 + 4) as f32, (height * 8 + 8) as f32);
        let level = Level {
            tiles: vec![[0, 0]; width * height],
            width,
            player_spawn,
        };
        let level_renderer = LevelRenderer::new(&level, assets, SKY_COLOR);

        let default_zoom_amt = 0.8;

        // all these calculations are literally just to zoom the viewport around the center of the level
        let screen = vec2(SCREEN_WIDTH, SCREEN_HEIGHT);
        let mut camera_zoom: f32 = 1.0;
        let mut camera_pos = level_renderer.size / 2.0 - screen / 2.0;
        let (mouse_x, mouse_y) = (screen.x / 2.0, screen.y / 2.0);
        let old_mouse_world_x = mouse_x + camera_pos.x - SCREEN_WIDTH / 2.0;
        let old_mouse_world_y = mouse_y + camera_pos.y - SCREEN_HEIGHT / 2.0;
        camera_zoom /= default_zoom_amt;
        camera_zoom = camera_zoom.max(MIN_ZOOM);
        camera_pos.x = old_mouse_world_x + SCREEN_WIDTH / 2.0 - mouse_x / camera_zoom;
        camera_pos.y = old_mouse_world_y + SCREEN_HEIGHT / 2.0 - mouse_y / camera_zoom;

        Self {
            level_renderer,
            assets,
            level,
            camera_zoom,
            camera_pos,
            sidebar: (0.0, 0, -1.0),
            dragging: Dragging::No,
        }
    }
    pub fn update(&mut self) -> bool {
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
        if !is_mouse_button_down(MouseButton::Left) {
            self.dragging = Dragging::No;
        }

        let mut scroll = mouse_wheel().1;
        let clicking = is_mouse_button_pressed(MouseButton::Left);

        self.sidebar.0 = (self.sidebar.0 + delta_time * self.sidebar.2 * 12.0).clamp(-0.0, 1.0);
        let sidebar_size = vec2(60.0, 275.0);

        let topbar = UIRect::new(
            vec2(1.0, 2.0) * scale_factor,
            vec2(actual_screen_width - 2.0 * scale_factor, 9.0 * scale_factor),
            MAKER_BG_COLOR,
            (scale_factor, BLACK),
        );

        let sidebar_pos = vec2(
            -sidebar_size.x + self.sidebar.0 * (sidebar_size.x + 1.0),
            12.0,
        );
        let sidebar_rect = UIRect::new(
            sidebar_pos * scale_factor,
            sidebar_size * scale_factor,
            MAKER_BG_COLOR,
            (scale_factor, BLACK),
        );
        let play_btn = UIImageButton::new(
            vec2(
                (actual_screen_width - self.assets.play_btn.frames[0].0.width() * scale_factor)
                    / 2.0,
                1.0 * scale_factor,
            ),
            &self.assets.play_btn.frames[0].0,
            &self.assets.play_btn.frames[1].0,
            scale_factor,
            false,
        );
        let handle_texture = if self.sidebar.2 < 0.0 {
            (
                &self.assets.handle_btn.frames[0].0,
                &self.assets.handle_btn.frames[1].0,
            )
        } else {
            (
                &self.assets.handle_btn.frames[2].0,
                &self.assets.handle_btn.frames[3].0,
            )
        };

        let start_play = (clicking && play_btn.is_hovered()) || is_key_pressed(KeyCode::R);
        let handle_btn = UIImageButton::new(
            (sidebar_pos + vec2(sidebar_size.x + 2.0, (sidebar_size.y - 9.0) / 2.0)) * scale_factor,
            handle_texture.0,
            handle_texture.1,
            scale_factor,
            false,
        );
        if clicking && handle_btn.is_hovered() {
            self.sidebar.2 *= -1.0;
        }

        let button_offset = vec2(19.0, 0.0);

        let mut tab_btns = Vec::new();
        for (i, t) in [
            &self.assets.tile_btn.frames,
            &self.assets.decoration_btn.frames,
            &self.assets.character_btn.frames,
        ]
        .iter()
        .enumerate()
        {
            let btn = UIImageButton::new(
                (sidebar_pos + 3.0 + button_offset * i as f32) * scale_factor,
                &t[0].0,
                &t[1].0,
                scale_factor,
                self.sidebar.1 == i as u8,
            );
            if btn.is_hovered() && clicking {
                self.sidebar.1 = i as u8;
            }
            tab_btns.push(btn);
        }

        let ui_hovered = topbar.is_hovered()
            || sidebar_rect.is_hovered()
            || handle_btn.is_hovered()
            || play_btn.is_hovered()
            || tab_btns.iter().any(|f| f.is_hovered());
        if ui_hovered && clicking {
            self.dragging = Dragging::UiOwned;
        } else if clicking {
            self.dragging = Dragging::WorldOwned;
        }

        let clicking_ui = matches!(self.dragging, Dragging::UiOwned);

        let mouse_delta = mouse_delta_position();
        let mut scroll_origin = vec2(mouse_x, mouse_y);
        if scroll == 0.0 {
            if is_key_pressed(KeyCode::Up) {
                scroll += 1.0;
                scroll_origin = vec2(
                    actual_screen_width / 2.0 / scale_factor,
                    actual_screen_height / 2.0 / scale_factor,
                );
            } else if is_key_pressed(KeyCode::Down) {
                scroll -= 1.0;
                scroll_origin = vec2(
                    actual_screen_width / 2.0 / scale_factor,
                    actual_screen_height / 2.0 / scale_factor,
                );
            }
        }

        // handle panning with middle-mouse
        if !clicking_ui && is_mouse_button_down(MouseButton::Middle) {
            self.camera_pos.x +=
                mouse_delta.x as f32 * actual_screen_width / scale_factor / 2.0 / self.camera_zoom;
            self.camera_pos.y +=
                mouse_delta.y as f32 * actual_screen_height / scale_factor / 2.0 / self.camera_zoom;
        }
        // handle scrolling
        if !clicking_ui && scroll != 0.0 {
            let amt = if scroll > 0.0 {
                1.0 / SCROLL_AMT
            } else {
                SCROLL_AMT
            };
            // store old mouse position (in world position)
            let old_mouse_world_x =
                scroll_origin.x / self.camera_zoom + self.camera_pos.x - SCREEN_WIDTH / 2.0;
            let old_mouse_world_y =
                scroll_origin.y / self.camera_zoom + self.camera_pos.y - SCREEN_HEIGHT / 2.0;

            // update grid size
            self.camera_zoom /= amt;
            self.camera_zoom = self.camera_zoom.max(MIN_ZOOM);
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            self.camera_pos.x =
                old_mouse_world_x + SCREEN_WIDTH / 2.0 - scroll_origin.x / self.camera_zoom;
            self.camera_pos.y =
                old_mouse_world_y + SCREEN_HEIGHT / 2.0 - scroll_origin.y / self.camera_zoom;
        }

        // handle clicking
        if !clicking_ui
            && is_mouse_button_down(MouseButton::Left)
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
        gl_use_material(&GRID_MATERIAL);
        GRID_MATERIAL.set_uniform("zoom", self.camera_zoom);
        GRID_MATERIAL.set_uniform("scale", scale_factor);
        GRID_MATERIAL.set_uniform("offset", self.camera_pos);
        GRID_MATERIAL.set_uniform("screen", vec2(actual_screen_width, actual_screen_height));

        draw_rectangle(0.0, 0.0, actual_screen_width, actual_screen_height, WHITE);
        gl_use_default_material();
        topbar.draw();
        sidebar_rect.draw();
        handle_btn.draw();
        for btn in tab_btns {
            btn.draw();
        }
        play_btn.draw();
        start_play
    }
}
