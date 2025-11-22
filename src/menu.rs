use crate::{assets::Assets, data::*, level::Level, ui::*, utils::*};
use macroquad::{miniquad::window::screen_size, prelude::*};

pub enum MenuUpdateResult {
    None,
    Create(Option<usize>),
    PlayOnline(Level, String, String),
}
enum LevelMenuType {
    Closed,
    BrowseOnline,
    LocalLevels,
}
enum PopupMenu {
    None,
    Delete(usize),
    Rename(usize, TextInputData),
    Upload(usize, TextInputData, TextInputData),
    Uploading(String),
    Downloading(String),
    Error(String),
}
impl PopupMenu {
    fn yes_button(&self) -> bool {
        match self {
            PopupMenu::Error(_) | PopupMenu::Uploading(_) | PopupMenu::Downloading(_) => false,
            _ => true,
        }
    }
}
pub struct MainMenu<'a> {
    assets: &'a Assets,
    level_menu: LevelMenuType,
    time: f32,
    popup: PopupMenu,
}
impl<'a> MainMenu<'a> {
    pub fn new(assets: &'a Assets) -> Self {
        Self {
            assets,
            level_menu: LevelMenuType::Closed,
            time: 0.0,
            popup: PopupMenu::None,
        }
    }
    pub fn update(&mut self, data: &mut Data) -> MenuUpdateResult {
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

        let received_upload_result = data.upload_result.take();
        let received_download_result = data.download_result.take();

        let mut mouse_down = is_mouse_button_pressed(MouseButton::Left);

        if !matches!(self.popup, PopupMenu::None) {
            mouse_down = false;
        }
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
            if matches!(self.level_menu, LevelMenuType::BrowseOnline) {
                if data.list_request.is_some() {
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
                if data.failed_list_request {
                    // if fetch request failed, show error icon
                    let pos = vec2(
                        menu_pos.x + (size.x / 2.0 - 151.0 / 2.0) * scale_factor,
                        menu_pos.y + 40.0 * scale_factor,
                    );
                    draw_texture_ex(
                        &self.assets.warning,
                        pos.x,
                        pos.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(40.0, 40.0) * scale_factor),
                            ..Default::default()
                        },
                    );
                    let font_size = (12.0 * scale_factor) as u16;
                    draw_text_ex(
                        "Server unreachable",
                        pos.x + 35.0 * scale_factor,
                        pos.y + font_size as f32,
                        TextParams {
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                }
            }

            let buttons_pos = menu_pos + vec2(3.0, 3.0) * scale_factor;
            let size = vec2(size.x - 6.0, 25.0);
            let offset = vec2(0.0, size.y + 5.0);

            let (names, title): (Vec<String>, &str) = match &self.level_menu {
                LevelMenuType::Closed => {
                    panic!()
                }
                LevelMenuType::BrowseOnline => (
                    data.online_levels
                        .iter()
                        .map(|f| f.split_once("-").unwrap().0.to_string())
                        .collect(),
                    "Online Levels",
                ),
                LevelMenuType::LocalLevels => (
                    data.local
                        .user_levels
                        .iter()
                        .map(|f| f.0.clone())
                        .rev()
                        .collect(),
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
                let btn = UIImageButton::new(
                    offset * scale_factor + buttons_pos,
                    &self.assets.create_btn.frames[0].0,
                    &self.assets.create_btn.frames[1].0,
                    scale_factor,
                    false,
                );
                btn.draw();
                if btn.is_hovered() && mouse_down {
                    return MenuUpdateResult::Create(None);
                }
            }

            for (i, name) in names.iter().enumerate() {
                let mut unallow_click = false;
                let mut item_buttons = Vec::new();
                if matches!(self.level_menu, LevelMenuType::LocalLevels) {
                    let item_offset = vec2(17.0, 0.0);
                    for (j, anim) in [
                        &self.assets.delete_btn,
                        &self.assets.rename_btn,
                        &self.assets.upload_btn,
                    ]
                    .iter()
                    .enumerate()
                    {
                        let btn = UIImageButton::new(
                            (offset * (i + 2) as f32 + size - 16.0 - 5.0 - item_offset * j as f32)
                                * scale_factor
                                + buttons_pos,
                            &anim.frames[0].0,
                            &anim.frames[1].0,
                            scale_factor,
                            false,
                        );
                        let hover = btn.is_hovered();
                        unallow_click |= hover;
                        item_buttons.push(btn);
                        if hover && mouse_down {
                            match j {
                                0 => {
                                    self.popup =
                                        PopupMenu::Delete(data.local.user_levels.len() - i - 1);
                                }
                                1 => {
                                    self.popup = PopupMenu::Rename(
                                        data.local.user_levels.len() - i - 1,
                                        TextInputData::from_text(
                                            data.local.user_levels
                                                [data.local.user_levels.len() - i - 1]
                                                .0
                                                .clone(),
                                        ),
                                    );
                                }
                                2 => {
                                    self.popup = PopupMenu::Upload(
                                        data.local.user_levels.len() - i - 1,
                                        TextInputData::from_text(
                                            data.local.user_levels
                                                [data.local.user_levels.len() - i - 1]
                                                .0
                                                .clone(),
                                        ),
                                        TextInputData::default(),
                                    );
                                }
                                _ => {
                                    panic!()
                                }
                            }
                        }
                    }
                }
                let btn = UITextButton::new(
                    (offset * (i + 2) as f32) * scale_factor + buttons_pos,
                    size * scale_factor,
                    name.to_string(),
                    SKY_COLOR,
                    if unallow_click {
                        SKY_COLOR
                    } else {
                        MAKER_BG_COLOR
                    },
                    (scale_factor, BLACK),
                    (
                        (12.5 * scale_factor) as u16,
                        &self.assets.font,
                        3.0 * scale_factor,
                    ),
                );
                btn.draw();
                if matches!(self.level_menu, LevelMenuType::BrowseOnline) {
                    draw_texture_ex(
                        &self.assets.person_icon,
                        btn.pos.x + 2.0 * scale_factor,
                        btn.pos.y + size.y * scale_factor - 8.0 * scale_factor,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(6.0, 6.0) * scale_factor),
                            ..Default::default()
                        },
                    );
                    let author = data.online_levels[i].split_once("-").unwrap().1;

                    let font_size = (10.0 * scale_factor) as u16;
                    draw_text_ex(
                        author,
                        btn.pos.x + 10.0 * scale_factor,
                        btn.pos.y + size.y * scale_factor - 4.0 * scale_factor,
                        TextParams {
                            color: LIGHTGRAY,
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                }
                for btn in item_buttons {
                    btn.draw();
                }

                if !unallow_click && btn.is_hovered() && mouse_down {
                    match self.level_menu {
                        LevelMenuType::BrowseOnline => {
                            let name = data.online_levels[i].to_string();
                            data.download_level(&name);
                            self.popup = PopupMenu::Downloading(name);
                            break;
                        }
                        LevelMenuType::LocalLevels => {
                            return MenuUpdateResult::Create(Some(
                                data.local.user_levels.len() - i - 1,
                            ));
                        }
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

        let popup_size = if !matches!(self.popup, PopupMenu::Upload(_, _, _)) {
            vec2(250.0, 105.0)
        } else {
            vec2(250.0, 150.0)
        };
        if !matches!(self.popup, PopupMenu::None) {
            let pos =
                (vec2(actual_screen_width, actual_screen_height) - popup_size * scale_factor) / 2.0;
            draw_rectangle(
                pos.x,
                pos.y,
                popup_size.x * scale_factor,
                popup_size.y * scale_factor,
                SKY_COLOR,
            );
            draw_rectangle_lines(
                pos.x,
                pos.y,
                popup_size.x * scale_factor,
                popup_size.y * scale_factor,
                2.0 * scale_factor,
                BLACK,
            );
            let font_size = (24.0 * scale_factor) as u16;
            match &mut self.popup {
                PopupMenu::Delete(_) => {
                    draw_text_ex(
                        "Are you sure?",
                        pos.x + 35.0 * scale_factor,
                        pos.y + font_size as f32,
                        TextParams {
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                }
                PopupMenu::Rename(_, data) => {
                    draw_text_ex(
                        "Rename",
                        pos.x + 10.0 * scale_factor,
                        pos.y + font_size as f32,
                        TextParams {
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                    let size = vec2(225.0, 25.0);
                    let font_size = (12.0 * scale_factor) as u16;
                    let mut input = UITextInput::new(
                        pos + vec2((popup_size.x - size.x) / 2.0, 36.0) * scale_factor,
                        size * scale_factor,
                        SKY_COLOR,
                        MAKER_BG_COLOR,
                        (scale_factor, BLACK),
                        (font_size, &self.assets.font, 3.0 * scale_factor),
                        data,
                        "Enter level name",
                        MAX_LEVEL_NAME_LENGTH,
                    );
                    input.draw();
                }
                PopupMenu::Upload(_, name_input, author_input) => {
                    draw_text_ex(
                        "Upload level",
                        pos.x + 10.0 * scale_factor,
                        pos.y + font_size as f32,
                        TextParams {
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                    let size = vec2(225.0, 25.0);
                    let font_size = (12.0 * scale_factor) as u16;
                    let mut input = UITextInput::new(
                        pos + vec2((popup_size.x - size.x) / 2.0, 36.0) * scale_factor,
                        size * scale_factor,
                        SKY_COLOR,
                        MAKER_BG_COLOR,
                        (scale_factor, BLACK),
                        (font_size, &self.assets.font, 3.0 * scale_factor),
                        name_input,
                        "Enter level name",
                        MAX_LEVEL_NAME_LENGTH,
                    );
                    input.draw();
                    let mut input = UITextInput::new(
                        pos + vec2((popup_size.x - size.x) / 2.0, 36.0 + size.y + 5.0)
                            * scale_factor,
                        size * scale_factor,
                        SKY_COLOR,
                        MAKER_BG_COLOR,
                        (scale_factor, BLACK),
                        (font_size, &self.assets.font, 3.0 * scale_factor),
                        author_input,
                        "Enter author name",
                        MAX_AUTHOR_NAME_LENGTH,
                    );
                    input.draw();
                }
                PopupMenu::Uploading(_) => {
                    draw_text_ex(
                        "Uploading...",
                        pos.x + 10.0 * scale_factor,
                        pos.y + font_size as f32,
                        TextParams {
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                    draw_texture_ex(
                        self.assets.spinner.get_at_time((self.time * 1000.0) as u32),
                        pos.x + 10.0 * scale_factor,
                        pos.y + font_size as f32 + 5.0 * scale_factor,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(40.0, 40.0) * scale_factor),
                            ..Default::default()
                        },
                    );
                    if let Some(result) = received_upload_result {
                        match result {
                            NetworkResult::Success => {
                                self.popup = PopupMenu::None;
                                data.update_level_list();
                                self.level_menu = LevelMenuType::BrowseOnline;
                            }
                            NetworkResult::Fail(e) => self.popup = PopupMenu::Error(e),
                        }
                    }
                }
                PopupMenu::Downloading(_) => {
                    draw_text_ex(
                        "Downloading level...",
                        pos.x + 10.0 * scale_factor,
                        pos.y + font_size as f32,
                        TextParams {
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                    draw_texture_ex(
                        self.assets.spinner.get_at_time((self.time * 1000.0) as u32),
                        pos.x + 10.0 * scale_factor,
                        pos.y + font_size as f32 + 5.0 * scale_factor,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(40.0, 40.0) * scale_factor),
                            ..Default::default()
                        },
                    );
                    if let Some((level, result)) = received_download_result {
                        match result {
                            NetworkResult::Success => {
                                self.popup = PopupMenu::None;
                                let (name, author) = level.split_once("-").unwrap();
                                let level = data.cached_online_levels.get(&level).unwrap().clone();
                                return MenuUpdateResult::PlayOnline(
                                    level,
                                    name.to_string(),
                                    author.to_string(),
                                );
                            }
                            NetworkResult::Fail(e) => self.popup = PopupMenu::Error(e),
                        }
                    }
                }
                PopupMenu::Error(text) => {
                    draw_text_ex(
                        "Error!",
                        pos.x + 10.0 * scale_factor,
                        pos.y + font_size as f32,
                        TextParams {
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                    let font_size = (12.0 * scale_factor) as u16;
                    draw_text_ex(
                        &text,
                        pos.x + (2.0) * scale_factor,
                        pos.y + (font_size) as f32 + 40.0 * scale_factor,
                        TextParams {
                            font_size,
                            font: Some(&self.assets.font),
                            ..Default::default()
                        },
                    );
                }
                _ => {}
            }
            let button_size = vec2(90.0, 25.0);
            let button_offset = vec2(button_size.x + 5.0, 0.0);

            if self.popup.yes_button() {
                let yes = UITextButton::new(
                    vec2(
                        pos.x + (popup_size.x / 2.0 - button_size.x - 5.0) * scale_factor,
                        pos.y + (popup_size.y - button_size.y - 9.0) * scale_factor,
                    ),
                    button_size * scale_factor,
                    "Yes".to_string(),
                    GREEN_COLOR,
                    DARK_GREEN_COLOR,
                    (scale_factor, BLACK),
                    (
                        (12.0 * scale_factor) as u16,
                        &self.assets.font,
                        3.0 * scale_factor,
                    ),
                );
                yes.draw();
                if yes.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
                    match &self.popup {
                        PopupMenu::Delete(index) => {
                            data.local.user_levels.remove(*index);
                            data.local.store();
                            self.popup = PopupMenu::None;
                        }
                        PopupMenu::Rename(index, text_data) => {
                            if !data
                                .local
                                .user_levels
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| index != i)
                                .any(|(_, f)| f.0 == text_data.text)
                            {
                                data.local.user_levels[*index].0 = text_data.text.clone();
                                self.popup = PopupMenu::None;
                                data.local.store();
                            }
                        }
                        PopupMenu::Upload(index, name_data, author_data) => {
                            if !data
                                .local
                                .user_levels
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| index != i)
                                .any(|(_, f)| f.0 == name_data.text)
                            {
                                data.local.user_levels[*index].0 = name_data.text.clone();
                                data.local.store();
                                data.upload_level(
                                    data.local.user_levels[*index].clone().1,
                                    name_data.text.to_string(),
                                    author_data.text.to_string(),
                                );
                                self.popup = PopupMenu::Uploading(name_data.text.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            let no = UITextButton::new(
                vec2(
                    pos.x
                        + (popup_size.x / 2.0 - button_size.x - 5.0) * scale_factor
                        + button_offset.x * scale_factor,
                    pos.y + (popup_size.y - button_size.y - 9.0) * scale_factor,
                ),
                button_size * scale_factor,
                "Cancel".to_string(),
                SKY_COLOR,
                MAKER_BG_COLOR,
                (scale_factor, BLACK),
                (
                    (12.0 * scale_factor) as u16,
                    &self.assets.font,
                    3.0 * scale_factor,
                ),
            );
            no.draw();
            if no.is_hovered() && is_mouse_button_pressed(MouseButton::Left) {
                self.popup = PopupMenu::None;
            }
        }

        if create_btn.is_hovered() && mouse_down {
            self.level_menu = LevelMenuType::LocalLevels;
            if data.local.user_levels.is_empty() {
                return MenuUpdateResult::Create(None);
            }
        } else if play_btn.is_hovered() && mouse_down {
            self.level_menu = LevelMenuType::BrowseOnline;
        }
        MenuUpdateResult::None
    }
}
