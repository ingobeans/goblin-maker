use std::collections::HashMap;

use asefile::AsepriteFile;
use image::EncodableLayout;
use macroquad::prelude::*;

pub struct Assets {
    pub player_torso: AnimationsGroup,
    pub player_legs: AnimationsGroup,

    pub terrain_tileset: Spritesheet,
    pub decoration_tileset: Spritesheet,
    pub character_tileset: Spritesheet,

    pub enemies: AnimationsGroup,

    // ui
    pub tile_btn: Animation,
    pub decoration_btn: Animation,
    pub character_btn: Animation,
    pub handle_btn: Animation,
    pub play_btn: Animation,
}
impl Default for Assets {
    fn default() -> Self {
        Self {
            enemies: AnimationsGroup::from_file(include_bytes!("../assets/enemies.ase")),

            player_torso: AnimationsGroup::from_file(include_bytes!("../assets/player_torso.ase")),
            player_legs: AnimationsGroup::from_file(include_bytes!("../assets/player_legs.ase")),

            terrain_tileset: Spritesheet::new(
                load_ase_texture(include_bytes!("../assets/terrain_tileset.ase"), None),
                16.0,
            )
            .autotile(
                load_ase_texture(include_bytes!("../assets/terrain.ase"), Some(0)),
                load_ase_texture(include_bytes!("../assets/terrain.ase"), Some(1)),
            ),
            decoration_tileset: Spritesheet::new(
                load_ase_texture(include_bytes!("../assets/decoration_tileset.ase"), Some(1)),
                16.0,
            ),
            character_tileset: Spritesheet::new(
                load_ase_texture(include_bytes!("../assets/character_tileset.ase"), Some(1)),
                16.0,
            ),

            tile_btn: Animation::from_file(include_bytes!("../assets/ui/tile_btn.ase")),
            decoration_btn: Animation::from_file(include_bytes!("../assets/ui/decoration_btn.ase")),
            character_btn: Animation::from_file(include_bytes!("../assets/ui/character_btn.ase")),
            handle_btn: Animation::from_file(include_bytes!("../assets/ui/handle_btn.ase")),
            play_btn: Animation::from_file(include_bytes!("../assets/ui/play_btn.ase")),
        }
    }
}
fn load_ase_texture(bytes: &[u8], layer: Option<u32>) -> Texture2D {
    let img = AsepriteFile::read(bytes).unwrap();
    let img = if let Some(layer) = layer {
        img.layer(layer).frame(0).image()
    } else {
        img.frame(0).image()
    };
    let new = Image {
        width: img.width() as u16,
        height: img.height() as u16,
        bytes: img.as_bytes().to_vec(),
    };
    let texture = Texture2D::from_image(&new);
    texture.set_filter(FilterMode::Nearest);
    texture
}
pub struct AnimationsGroup {
    #[expect(dead_code)]
    pub file: AsepriteFile,
    pub animations: Vec<Animation>,
    pub tag_names: HashMap<String, usize>,
}
impl AnimationsGroup {
    pub fn get_by_name(&self, name: &str) -> &Animation {
        &self.animations[*self.tag_names.get(name).unwrap()]
    }
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            let texture = Texture2D::from_image(&new);
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, duration));
        }
        let mut tag_frames = Vec::new();
        let mut offset = 0;

        let mut tag_names = HashMap::new();

        for i in 0..ase.num_tags() {
            let tag = ase.get_tag(i).unwrap();
            tag_names.insert(tag.name().to_string(), i as usize);
            let (start, end) = (tag.from_frame() as usize, tag.to_frame() as usize);
            let mut total_length = 0;
            let included_frames: Vec<(Texture2D, u32)> = frames
                .extract_if((start - offset)..(end - offset + 1), |_| true)
                .collect();
            for f in included_frames.iter() {
                total_length += f.1;
            }
            offset += end.abs_diff(start) + 1;
            tag_frames.push(Animation {
                frames: included_frames,
                total_length,
            });
        }
        Self {
            file: ase,
            animations: tag_frames,
            tag_names,
        }
    }
}

pub struct Spritesheet {
    pub texture: Texture2D,
    pub sprite_size: f32,
    /// Special case, if true, first tile of the spritesheet is autotiled.
    pub autotile_first: Option<(HashMap<[bool; 4], Vec2>, Box<Spritesheet>)>,
}
impl Spritesheet {
    pub fn new(texture: Texture2D, sprite_size: f32) -> Self {
        Self {
            texture,
            sprite_size,
            autotile_first: None,
        }
    }
    pub fn autotile(
        mut self,
        autotiling_tileset: Texture2D,
        autotiling_ruleset: Texture2D,
    ) -> Self {
        let mut hashmap = HashMap::new();
        let image = autotiling_ruleset.get_texture_data();
        for x in 0..(autotiling_tileset.width() as u32) / 16 {
            let x = x * 16;
            for y in 0..(autotiling_tileset.height() as u32) / 16 {
                let y = y * 16;
                let [r, g, b, a] = image.bytes[x as usize * 4
                    + y as usize * 4 * image.width as usize
                    ..x as usize * 4 + y as usize * 4 * image.width as usize + 4]
                else {
                    panic!("._.")
                };
                if a == 0 {
                    continue;
                }
                hashmap.insert(
                    [r == 255, g == 255, b == 255, a == 255],
                    vec2(x as f32, y as f32),
                );
            }
        }
        self.autotile_first = Some((
            hashmap,
            Box::new(Spritesheet::new(autotiling_tileset, 16.0)),
        ));
        self
    }
    #[expect(dead_code)]
    /// Same as `draw_tile`, except centered
    pub fn draw_sprite(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        self.draw_tile(
            screen_x - self.sprite_size / 2.0,
            screen_y - self.sprite_size / 2.0,
            tile_x,
            tile_y,
            params,
        );
    }
    /// Draws a single tile from the spritesheet
    pub fn draw_tile(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        let mut p = params.cloned().unwrap_or(DrawTextureParams::default());
        p.dest_size = p
            .dest_size
            .or(Some(Vec2::new(self.sprite_size, self.sprite_size)));
        p.source = p.source.or(Some(Rect {
            x: tile_x * self.sprite_size,
            y: tile_y * self.sprite_size,
            w: self.sprite_size,
            h: self.sprite_size,
        }));
        draw_texture_ex(&self.texture, screen_x, screen_y, WHITE, p);
    }
}

pub struct Animation {
    pub frames: Vec<(Texture2D, u32)>,
    pub total_length: u32,
}
impl Animation {
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        let mut total_length = 0;
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            total_length += duration;
            let texture = Texture2D::from_image(&new);
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, duration));
        }
        Self {
            frames,
            total_length,
        }
    }
    pub fn get_at_time(&self, mut time: u32) -> &Texture2D {
        time %= self.total_length;
        for (texture, length) in self.frames.iter() {
            if time >= *length {
                time -= length;
            } else {
                return texture;
            }
        }
        panic!()
    }
}
