use impl_new_derive::ImplNew;
use macroquad::prelude::*;

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
