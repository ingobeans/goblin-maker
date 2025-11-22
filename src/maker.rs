use crate::{
    assets::{Assets, Spritesheet},
    level::{Character, Level, LevelRenderer},
    ui::*,
    utils::*,
};
use enum_iterator::{Sequence, all};
use macroquad::{miniquad::window::screen_size, prelude::*};

enum Dragging {
    No,
    UiOwned,
    WorldOwned(u8),
    MenuOwned,
}

#[derive(Sequence, PartialEq, Eq)]
enum Tool {
    Pencil,
    Eraser,
    Bucket,
}

fn get_connected_tiles(level: &Level, tx: usize, ty: usize) -> Vec<(usize, usize)> {
    let mut active = vec![(tx, ty)];
    let mut tiles = vec![(tx, ty)];

    while let Some((tx, ty)) = active.pop() {
        let mut tile_positions = vec![(tx + 1, ty), (tx, ty + 1)];
        if ty > 0 {
            tile_positions.push((tx, ty.saturating_sub(1)));
        }
        if tx > 0 {
            tile_positions.push((tx.saturating_sub(1), ty));
        }
        for (tx, ty) in tile_positions {
            if tx >= level.width || ty >= level.height() {
                continue;
            }
            if tiles.contains(&(tx, ty)) {
                continue;
            }
            if level.tiles[tx + ty * level.width] == [0, 0] {
                tiles.push((tx, ty));
                active.push((tx, ty));
            }
        }
    }
    tiles
}

pub struct GoblinMaker<'a> {
    assets: &'a Assets,
    pub name: Option<String>,
    pub level: Level,
    level_renderer: LevelRenderer<'a>,
    camera_pos: Vec2,
    camera_zoom: f32,
    sidebar: (f32, u8, f32),
    dragging: Dragging,
    selected_tile: Option<(usize, u8)>,
    tab_tiles: [(&'a Spritesheet, Vec<Vec2>); 3],
    tool: Tool,
}

fn get_tab_tiles(assets: &Assets) -> [(&Spritesheet, Vec<Vec2>); 3] {
    fn get_tiles(tab: usize, assets: &Assets) -> (&Spritesheet, Vec<Vec2>) {
        let texture = if tab == 0 {
            &assets.terrain_tileset
        } else if tab == 1 {
            &assets.decoration_tileset
        } else {
            &assets.character_tileset
        };

        let image = texture.texture.get_texture_data();
        let mut tiles = Vec::new();
        'outer: for y in 0..texture.texture.height() as u32 / 16 {
            let y = y as f32 * 16.0;
            for x in 0..texture.texture.width() as u32 / 16 {
                let x = x as f32 * 16.0;
                let area = image.sub_image(Rect {
                    x,
                    y,
                    w: 16.0,
                    h: 16.0,
                });
                let mut empty = true;
                for pixel in area.get_image_data().iter() {
                    if pixel[3] != 0 {
                        empty = false;
                        break;
                    }
                }
                if empty {
                    break 'outer;
                }
                tiles.push(vec2(x / 16.0, y / 16.0));
            }
        }
        (texture, tiles)
    }
    std::array::from_fn(|f| get_tiles(f, assets))
}

impl<'a> GoblinMaker<'a> {
    pub fn from(assets: &'a Assets, level: Level, name: Option<String>) -> Self {
        let default_zoom_amt = 0.8;
        let level_renderer = LevelRenderer::new(&level, assets, SKY_COLOR);
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
            name,
            tab_tiles: get_tab_tiles(assets),
            level_renderer,
            assets,
            level,
            camera_zoom,
            camera_pos,
            sidebar: (999.0, 0, 1.0),
            dragging: Dragging::MenuOwned,
            selected_tile: Some((0, 0)),
            tool: Tool::Pencil,
        }
    }
    pub fn new(assets: &'a Assets) -> Self {
        let width = 100;
        let height = 50;
        let player_spawn = vec2((width * 8 + 4) as f32, (height * 8 + 8) as f32);

        let player_pos = vec2(player_spawn.x - 4.0, player_spawn.y - 8.0);
        let level = Level {
            tiles: vec![[0, 0]; width * height],
            width,
            characters: vec![((player_pos.x, player_pos.y), Character::PlayerSpawn, 0)],
        };

        Self::from(assets, level, None)
    }
    fn use_tool(&mut self, tx: usize, ty: usize, tile_index: usize, tab_index: u8) {
        match self.tool {
            Tool::Pencil => {
                if tab_index == 2 {
                    // tab index 2 is character tab. place character
                    if is_mouse_button_pressed(MouseButton::Left) {
                        let pos = ((tx * 16) as f32, (ty * 16) as f32);
                        // check no character is already placed there
                        if !self.level.characters.iter().any(|f| f.0 == pos) {
                            let character = match tile_index {
                                0 => Character::PlayerSpawn,
                                1 => Character::Checkpoint,
                                _ => Character::WanderEnemy(tile_index - 2),
                            };
                            let bundle = (pos, character, tile_index);
                            if tile_index == 0 {
                                self.level.characters[0] = bundle;
                            } else {
                                self.level.characters.push(bundle);
                            }
                        }
                    }
                } else {
                    // general tile placing code
                    let mut tile = self.level.get_tile(tx, ty);
                    tile[tab_index as usize] = tile_index as u8 + 1;
                    self.level_renderer.set_tile(&mut self.level, tx, ty, tile);
                }
            }
            Tool::Eraser => {
                if self.sidebar.1 == 2 {
                    let pos = ((tx * 16) as f32, (ty * 16) as f32);

                    self.level.characters.retain(|f| f.0 != pos);
                } else {
                    // general tile placing code
                    let mut tile = self.level.get_tile(tx, ty);
                    let Dragging::WorldOwned(layer) = self.dragging else {
                        panic!()
                    };
                    tile[layer as usize] = 0;
                    self.level_renderer.set_tile(&mut self.level, tx, ty, tile);
                }
            }
            Tool::Bucket => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let tiles = get_connected_tiles(&self.level, tx, ty);
                    for (tx, ty) in tiles.into_iter() {
                        let mut tile = self.level.get_tile(tx, ty);
                        tile[tab_index as usize] = tile_index as u8 + 1;
                        self.level_renderer.set_tile(&mut self.level, tx, ty, tile);
                    }
                }
            }
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

        let tab = &self.tab_tiles[self.sidebar.1 as usize];
        let mut tile_btns = Vec::new();
        for (index, tile) in tab.1.iter().enumerate() {
            let selected = if let Some(selected) = &self.selected_tile
                && selected.0 == index
                && selected.1 == self.sidebar.1
            {
                WHITE
            } else {
                BLACK
            };

            let button = UITileButton::new(
                (vec2((index % 3) as f32 * 19.0, (index / 3) as f32 * 19.0)
                    + sidebar_pos
                    + vec2(3.0, 25.0))
                    * scale_factor,
                tab.0,
                *tile,
                scale_factor,
                SKY_COLOR,
                selected,
            );
            if button.is_hovered() && clicking {
                self.selected_tile = Some((index, self.sidebar.1));
            }
            tile_btns.push(button);
        }
        let mut tool_btns = Vec::new();
        let button_offset = vec2(11.0, 0.0);

        for (index, (tool, animation)) in
            all::<Tool>().zip(self.assets.tool_btns.iter()).enumerate()
        {
            let active = self.tool == tool;
            let t = if active {
                &animation.frames[0].0
            } else {
                &animation.frames[1].0
            };
            let btn = UIImageButton::new(
                (vec2(5.0, 2.0) + button_offset * index as f32) * scale_factor,
                t,
                t,
                scale_factor,
                active,
            );
            if btn.is_hovered() && clicking {
                self.tool = tool;
            }
            tool_btns.push(btn);
        }
        if is_key_pressed(KeyCode::E) {
            self.tool = Tool::Eraser;
        }
        if is_key_pressed(KeyCode::B) || is_key_pressed(KeyCode::P) {
            self.tool = Tool::Pencil;
        }
        if is_key_pressed(KeyCode::F) || is_key_pressed(KeyCode::G) {
            self.tool = Tool::Bucket;
        }

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
            || tab_btns.iter().any(|f| f.is_hovered())
            || tool_btns.iter().any(|f| f.is_hovered())
            || tile_btns.iter().any(|f| f.is_hovered());
        if ui_hovered && clicking {
            self.dragging = Dragging::UiOwned;
        } else if clicking {
            // find what layer it is we are clicking.

            let tile = cursor_tile.map(|(tx, ty)| self.level.get_tile(tx, ty));
            let layer = if let Some(tile) = tile {
                if (tile[0] == 0 && tile[1] == 0) || (tile[1] != 0 && tile[0] != 0) {
                    self.sidebar.1
                } else if tile[0] == 0 {
                    1
                } else {
                    0
                }
            } else {
                self.sidebar.1
            };
            self.dragging = Dragging::WorldOwned(layer);
        }

        let clicking_ui = matches!(self.dragging, Dragging::UiOwned);
        let allow_world_mouse = matches!(self.dragging, Dragging::WorldOwned(_));

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
        if allow_world_mouse
            && is_mouse_button_down(MouseButton::Left)
            && let Some((tx, ty)) = cursor_tile
            && let Some((tile_index, tab_index)) = self.selected_tile
        {
            self.use_tool(tx, ty, tile_index, tab_index);
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
        let params = DrawTextureParams {
            dest_size: Some(vec2(
                16.0 * scale_factor * self.camera_zoom,
                16.0 * scale_factor * self.camera_zoom,
            )),
            ..Default::default()
        };
        for (pos, _, index) in self.level.characters.iter() {
            self.assets.character_tileset.draw_tile(
                (pos.0) * scale_factor * self.camera_zoom
                    - self.camera_pos.x * scale_factor * self.camera_zoom,
                (pos.1) * scale_factor * self.camera_zoom
                    - self.camera_pos.y * scale_factor * self.camera_zoom,
                (index % 3) as f32,
                (index / 3) as f32,
                Some(&params),
            );
        }

        match &self.tool {
            Tool::Pencil => {
                if let Some((index, tab)) = self.selected_tile
                    && let Some((tx, ty)) = cursor_tile
                    && {
                        if tab == 2 {
                            true
                        } else {
                            self.level.get_tile(tx, ty)[tab as usize] != (index + 1) as u8
                        }
                    }
                {
                    let tileset = [
                        &self.assets.terrain_tileset,
                        &self.assets.decoration_tileset,
                        &self.assets.character_tileset,
                    ][tab as usize];
                    let pos = vec2((tx * 16) as f32, (ty * 16) as f32);
                    let mut params = params.clone();
                    params.source = Some(Rect {
                        x: (index % 3) as f32 * 16.0,
                        y: (index / 3) as f32 * 16.0,
                        w: 16.0,
                        h: 16.0,
                    });
                    draw_texture_ex(
                        &tileset.texture,
                        (pos.x) * scale_factor * self.camera_zoom
                            - self.camera_pos.x * scale_factor * self.camera_zoom,
                        (pos.y) * scale_factor * self.camera_zoom
                            - self.camera_pos.y * scale_factor * self.camera_zoom,
                        WHITE.with_alpha(0.75),
                        params,
                    );
                }
            }
            Tool::Eraser => {
                if let Some((tx, ty)) = cursor_tile {
                    let pos = vec2((tx * 16) as f32, (ty * 16) as f32);
                    draw_rectangle_lines(
                        (pos.x) * scale_factor * self.camera_zoom
                            - self.camera_pos.x * scale_factor * self.camera_zoom,
                        (pos.y) * scale_factor * self.camera_zoom
                            - self.camera_pos.y * scale_factor * self.camera_zoom,
                        16.0 * scale_factor * self.camera_zoom,
                        16.0 * scale_factor * self.camera_zoom,
                        1.0 * scale_factor * self.camera_zoom,
                        BLACK,
                    );
                    draw_rectangle_lines(
                        (pos.x + 0.5) * scale_factor * self.camera_zoom
                            - self.camera_pos.x * scale_factor * self.camera_zoom,
                        (pos.y + 0.5) * scale_factor * self.camera_zoom
                            - self.camera_pos.y * scale_factor * self.camera_zoom,
                        15.0 * scale_factor * self.camera_zoom,
                        15.0 * scale_factor * self.camera_zoom,
                        1.0 * scale_factor * self.camera_zoom,
                        WHITE,
                    );
                }
            }
            _ => {}
        }
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
        for btn in tile_btns {
            btn.draw();
        }
        for btn in tool_btns {
            btn.draw();
        }
        play_btn.draw();
        start_play
    }
}
