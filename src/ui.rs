use impl_new_derive::ImplNew;
use macroquad::prelude::*;

use crate::assets::Spritesheet;
#[derive(ImplNew)]
pub struct UIImageButton<'a> {
    pub pos: Vec2,
    pub texture: &'a Texture2D,
    pub hovered: &'a Texture2D,
    pub scale_factor: f32,
    pub show_pressed: bool,
}
impl<'a> UIImageButton<'a> {
    pub fn is_hovered(&self) -> bool {
        let size = self.texture.size() * self.scale_factor;
        let mouse = mouse_position();
        (self.pos.x..self.pos.x + size.x).contains(&mouse.0)
            && (self.pos.y..self.pos.y + size.y).contains(&mouse.1)
    }
    pub fn draw(&self) {
        let texture = if self.show_pressed || self.is_hovered() {
            self.hovered
        } else {
            self.texture
        };
        draw_texture_ex(
            texture,
            self.pos.x.floor(),
            self.pos.y.floor(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.scale_factor * texture.size()),
                ..Default::default()
            },
        );
    }
}

#[derive(Default)]
pub struct TextInputData {
    pub selected: bool,
    pub cursor_pos: usize,
    pub text: String,
}
impl TextInputData {
    pub fn from_text(text: String) -> Self {
        let mut data = Self::default();
        data.cursor_pos = text.len();
        data.text = text;
        data
    }
}

#[derive(ImplNew)]
pub struct UITextInput<'a> {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub hover_color: Color,
    pub border: (f32, Color),
    pub font: (u16, &'a Font, f32),
    pub data: &'a mut TextInputData,
    pub placeholder: &'static str,
    pub max_length: usize,
}
impl<'a> UITextInput<'a> {
    pub fn is_hovered(&self) -> bool {
        let mouse = mouse_position();
        (self.pos.x..self.pos.x + self.size.x).contains(&mouse.0)
            && (self.pos.y..self.pos.y + self.size.y).contains(&mouse.1)
    }
    pub fn draw(&mut self) {
        let hovered = self.is_hovered();
        let color = if self.data.selected || hovered {
            self.hover_color
        } else {
            self.color
        };
        if is_mouse_button_pressed(MouseButton::Left) {
            self.data.selected = hovered;
        }

        // handle key events
        if self.data.selected {
            let mut backspace_pressed = false;
            if let Some(key) = get_char_pressed() {
                if (key.is_alphanumeric() && key.is_ascii() || key == ' ')
                    && self.data.text.len() < self.max_length
                {
                    self.data.text.insert(self.data.cursor_pos, key);
                    self.data.cursor_pos += 1;
                } else if key == '\u{8}' {
                    backspace_pressed = true
                }
            } else if is_key_pressed(KeyCode::Left) {
                self.data.cursor_pos = self.data.cursor_pos.saturating_sub(1);
            } else if is_key_pressed(KeyCode::Right) {
                self.data.cursor_pos = (self.data.cursor_pos + 1).min(self.data.text.len());
            } else if is_key_pressed(KeyCode::Delete) {
                if self.data.cursor_pos < self.data.text.len() {
                    self.data.text.remove(self.data.cursor_pos);
                }
            } else if is_key_pressed(KeyCode::Home) {
                self.data.cursor_pos = 0;
            } else if is_key_pressed(KeyCode::End) {
                self.data.cursor_pos = self.data.text.len();
            } else if is_key_pressed(KeyCode::Backspace) {
                backspace_pressed = true;
            }

            if backspace_pressed && self.data.cursor_pos > 0 {
                self.data.cursor_pos = self.data.cursor_pos.saturating_sub(1);
                self.data.text.remove(self.data.cursor_pos);
            }
        }

        draw_rectangle(
            self.pos.x.floor(),
            self.pos.y.floor(),
            self.size.x.floor(),
            self.size.y.floor(),
            color,
        );
        draw_rectangle_lines(
            self.pos.x.floor(),
            self.pos.y.floor(),
            self.size.x.floor(),
            self.size.y.floor(),
            self.border.0.floor() * 2.0,
            self.border.1,
        );
        if self.data.selected {
            let (before, after) = self.data.text.split_at(self.data.cursor_pos);
            let pos = draw_text_ex(
                before,
                self.pos.x + self.font.2,
                self.pos.y + self.font.0 as f32,
                TextParams {
                    font_size: self.font.0,
                    font: Some(self.font.1),
                    ..Default::default()
                },
            );
            let pos2 = draw_text_ex(
                "|",
                self.pos.x + self.font.2 + pos.width,
                self.pos.y + self.font.0 as f32,
                TextParams {
                    font_size: self.font.0,
                    font: Some(self.font.1),
                    ..Default::default()
                },
            );
            draw_text_ex(
                after,
                self.pos.x + self.font.2 + pos.width + pos2.width,
                self.pos.y + self.font.0 as f32,
                TextParams {
                    font_size: self.font.0,
                    font: Some(self.font.1),
                    ..Default::default()
                },
            );
        } else {
            let (text, color): (&str, Color) = if self.data.text.is_empty() {
                (self.placeholder, LIGHTGRAY)
            } else {
                (&self.data.text, WHITE)
            };
            draw_text_ex(
                text,
                self.pos.x + self.font.2,
                self.pos.y + self.font.0 as f32,
                TextParams {
                    font_size: self.font.0,
                    font: Some(self.font.1),
                    color,
                    ..Default::default()
                },
            );
        }
    }
}
#[derive(ImplNew)]
pub struct UITextButton<'a> {
    pub pos: Vec2,
    pub size: Vec2,
    pub text: String,
    pub color: Color,
    pub hover_color: Color,
    pub border: (f32, Color),
    pub font: (u16, &'a Font, f32),
}
impl<'a> UITextButton<'a> {
    pub fn is_hovered(&self) -> bool {
        let mouse = mouse_position();
        (self.pos.x..self.pos.x + self.size.x).contains(&mouse.0)
            && (self.pos.y..self.pos.y + self.size.y).contains(&mouse.1)
    }
    pub fn draw(&self) {
        let color = if self.is_hovered() {
            self.hover_color
        } else {
            self.color
        };
        draw_rectangle(
            self.pos.x.floor(),
            self.pos.y.floor(),
            self.size.x.floor(),
            self.size.y.floor(),
            color,
        );
        draw_rectangle_lines(
            self.pos.x.floor(),
            self.pos.y.floor(),
            self.size.x.floor(),
            self.size.y.floor(),
            self.border.0.floor() * 2.0,
            self.border.1,
        );
        draw_text_ex(
            &self.text,
            self.pos.x + self.font.2,
            self.pos.y + self.font.0 as f32,
            TextParams {
                font_size: self.font.0,
                font: Some(self.font.1),
                ..Default::default()
            },
        );
    }
}

#[derive(ImplNew)]
pub struct UITileButton<'a> {
    pub pos: Vec2,
    pub tileset: &'a Spritesheet,
    pub tile: Vec2,
    pub scale_factor: f32,
    pub color: Color,
    pub border_color: Color,
}
impl<'a> UITileButton<'a> {
    pub fn is_hovered(&self) -> bool {
        let size = 16.0 * self.scale_factor;
        let mouse = mouse_position();
        (self.pos.x..self.pos.x + size).contains(&mouse.0)
            && (self.pos.y..self.pos.y + size).contains(&mouse.1)
    }
    pub fn draw(&self) {
        draw_rectangle(
            self.pos.x.floor() - self.scale_factor,
            self.pos.y.floor() - self.scale_factor,
            18.0 * self.scale_factor,
            18.0 * self.scale_factor,
            self.border_color,
        );
        draw_rectangle(
            self.pos.x.floor(),
            self.pos.y.floor(),
            16.0 * self.scale_factor,
            16.0 * self.scale_factor,
            self.color,
        );
        self.tileset.draw_tile(
            self.pos.x.floor(),
            self.pos.y.floor(),
            self.tile.x,
            self.tile.y,
            Some(&DrawTextureParams {
                dest_size: Some(self.scale_factor * vec2(16.0, 16.0)),
                ..Default::default()
            }),
        );
    }
}

#[derive(ImplNew)]
pub struct UIRect {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub border: (f32, Color),
}
impl UIRect {
    pub fn is_hovered(&self) -> bool {
        let mouse = mouse_position();
        (self.pos.x..self.pos.x + self.size.x).contains(&mouse.0)
            && (self.pos.y..self.pos.y + self.size.y).contains(&mouse.1)
    }
    pub fn draw(&self) {
        draw_rectangle(
            self.pos.x.floor(),
            self.pos.y.floor(),
            self.size.x.floor(),
            self.size.y.floor(),
            self.color,
        );
        draw_rectangle_lines(
            self.pos.x.floor(),
            self.pos.y.floor(),
            self.size.x.floor(),
            self.size.y.floor(),
            self.border.0.floor() * 2.0,
            self.border.1,
        );
    }
}
