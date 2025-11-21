use crate::{assets::Assets, ui::*, utils::*};
use macroquad::{miniquad::window::screen_size, prelude::*};

pub struct MainMenu<'a> {
    assets: &'a Assets,
}
pub enum MenuUpdateResult {
    None,
    Create,
    Play,
}
impl<'a> MainMenu<'a> {
    pub fn new(assets: &'a Assets) -> Self {
        Self { assets }
    }
    pub fn update(&mut self) -> MenuUpdateResult {
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        clear_background(SKY_COLOR);
        gl_use_material(&GRID_MATERIAL);
        GRID_MATERIAL.set_uniform("zoom", 1.0_f32);
        GRID_MATERIAL.set_uniform("scale", scale_factor);
        GRID_MATERIAL.set_uniform("offset", Vec2::ZERO);
        GRID_MATERIAL.set_uniform("screen", vec2(actual_screen_width, actual_screen_height));
        draw_rectangle(0.0, 0.0, actual_screen_width, actual_screen_height, WHITE);
        gl_use_default_material();

        let buttons_start = vec2(22.0, 113.0);
        let play_btn = UIImageButton::new(
            buttons_start * scale_factor,
            &self.assets.menu_play_btn.frames[0].0,
            &self.assets.menu_play_btn.frames[1].0,
            scale_factor,
            false,
        );
        let create_btn = UIImageButton::new(
            (buttons_start + vec2(0.0, 36.0)) * scale_factor,
            &self.assets.menu_create_btn.frames[0].0,
            &self.assets.menu_create_btn.frames[1].0,
            scale_factor,
            false,
        );

        draw_texture_ex(
            &self.assets.logo,
            22.0 * scale_factor,
            7.0 * scale_factor,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.assets.logo.size() * scale_factor),
                ..Default::default()
            },
        );
        play_btn.draw();
        create_btn.draw();
        if create_btn.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
            MenuUpdateResult::Create
        } else if play_btn.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
            MenuUpdateResult::Play
        } else {
            MenuUpdateResult::None
        }
    }
}
