use crate::{assets::Assets, data::*, ui::*, utils::*};
use macroquad::{miniquad::window::screen_size, prelude::*};

pub enum MenuUpdateResult {
    None,
    Create(Option<usize>),
    PlayOnline(usize),
}
enum LevelMenuType {
    Closed,
    BrowseOnline,
    LocalLevels,
}
pub struct MainMenu<'a> {
    assets: &'a Assets,
    level_menu: LevelMenuType,
    time: f32,
}
impl<'a> MainMenu<'a> {
    pub fn new(assets: &'a Assets) -> Self {
        Self {
            assets,
            level_menu: LevelMenuType::Closed,
            time: 0.0,
        }
    }
    pub fn update(&mut self, data: &Data) -> MenuUpdateResult {
        let delta_time = get_frame_time();
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

        self.time += delta_time;

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

        if !matches!(self.level_menu, LevelMenuType::Closed) {
            let size = vec2(156.0, 225.0);
            let menu_pos = vec2(
                actual_screen_width - 22.0 - size.x * scale_factor,
                26.0 * scale_factor,
            );
            let rect = UIRect::new(
                menu_pos,
                size * scale_factor,
                MAKER_BG_COLOR,
                (scale_factor, BLACK),
            );
            rect.draw();
            if matches!(self.level_menu, LevelMenuType::BrowseOnline) && data.list_request.is_some()
            {
                // if fetch request is active, show spinner
                draw_texture_ex(
                    self.assets.spinner.get_at_time((self.time * 1000.0) as u32),
                    menu_pos.x + (size.x / 2.0 - 20.0) * scale_factor,
                    menu_pos.y + 40.0 * scale_factor,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(40.0, 40.0) * scale_factor),
                        ..Default::default()
                    },
                );
            }

            let buttons_pos = menu_pos + vec2(3.0, 3.0) * scale_factor;
            let size = vec2(size.x - 6.0, 25.0);
            let offset = vec2(0.0, size.y + 5.0);

            let (names, title): (Vec<&String>, &str) = match &self.level_menu {
                LevelMenuType::Closed => {
                    panic!()
                }
                LevelMenuType::BrowseOnline => {
                    (data.online_levels.iter().collect(), "Online Levels")
                }
                LevelMenuType::LocalLevels => (
                    data.local.user_levels.iter().map(|f| &f.0).rev().collect(),
                    "My Levels",
                ),
            };

            let font_size = (16.0 * scale_factor) as u16;
            draw_text_ex(
                title,
                buttons_pos.x + 3.0 * scale_factor,
                buttons_pos.y + font_size as f32,
                TextParams {
                    font_size,
                    font: Some(&self.assets.font),
                    ..Default::default()
                },
            );
            if matches!(self.level_menu, LevelMenuType::LocalLevels) {
                let btn = UITextButton::new(
                    offset * scale_factor + buttons_pos,
                    size * scale_factor,
                    "CREATE NEW".to_string(),
                    SKY_COLOR,
                    MAKER_BG_COLOR,
                    (scale_factor, BLACK),
                    (
                        (12.5 * scale_factor) as u16,
                        &self.assets.font,
                        3.0 * scale_factor,
                    ),
                );
                btn.draw();
                if btn.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
                    return MenuUpdateResult::Create(None);
                }
            }

            for (i, name) in names.iter().enumerate() {
                let btn = UITextButton::new(
                    (offset * (i + 2) as f32) * scale_factor + buttons_pos,
                    size * scale_factor,
                    name.to_string(),
                    SKY_COLOR,
                    MAKER_BG_COLOR,
                    (scale_factor, BLACK),
                    (
                        (12.5 * scale_factor) as u16,
                        &self.assets.font,
                        3.0 * scale_factor,
                    ),
                );
                btn.draw();

                if btn.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
                    match self.level_menu {
                        LevelMenuType::BrowseOnline => return MenuUpdateResult::PlayOnline(i),
                        LevelMenuType::LocalLevels => return MenuUpdateResult::Create(Some(i)),
                        _ => {}
                    }
                }
            }
        }

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
            self.level_menu = LevelMenuType::LocalLevels;
            if data.local.user_levels.is_empty() {
                return MenuUpdateResult::Create(None);
            }
        } else if play_btn.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
            self.level_menu = LevelMenuType::BrowseOnline;
        }
        MenuUpdateResult::None
    }
}
