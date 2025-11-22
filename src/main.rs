use macroquad::prelude::*;

use crate::{
    assets::Assets,
    data::Data,
    maker::*,
    menu::{MainMenu, MenuUpdateResult},
    runtime::*,
    utils::*,
};

mod assets;
mod data;
mod level;
mod maker;
mod menu;
mod player;
mod runtime;
mod ui;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct GameManager<'a> {
    assets: &'a Assets,
    data: Data,
    menu: MainMenu<'a>,
    runtime: Option<GoblinRuntime<'a>>,
    maker: Option<GoblinMaker<'a>>,
}

impl<'a> GameManager<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self {
            data: Data::load(),
            menu: MainMenu::new(assets),
            maker: None,
            assets,
            runtime: None,
        }
    }
    fn update(&mut self) {
        self.data.update();
        if is_key_pressed(KeyCode::Escape)
            && let Some(maker) = self.maker.take()
        {
            let level = maker.level;
            let name = if let Some(name) = maker.name {
                name
            } else {
                let mut i = 0;
                let mut name;
                loop {
                    i += 1;
                    name = format!("Unnamed {i}");
                    if !self.data.local.user_levels.iter().any(|f| f.0 == name) {
                        break;
                    }
                }
                name
            };
            if let Some(old) = self.data.local.user_levels.iter_mut().find(|f| f.0 == name) {
                old.1 = level;
            } else {
                self.data.local.user_levels.push((name, level));
            }
            self.data.local.store();
        }
        if let Some(runtime) = &mut self.runtime {
            let result = runtime.update();
            if result {
                self.runtime = None;
            }
        } else if let Some(maker) = &mut self.maker {
            let result = maker.update();
            if result {
                self.runtime = Some(GoblinRuntime::new(self.assets, maker.level.clone(), None));
            }
        } else {
            // neither runtime or maker is open, draw main menu
            let result = self.menu.update(&mut self.data);
            match result {
                MenuUpdateResult::Create(value) => {
                    if let Some(index) = value {
                        let data = self.data.local.user_levels[index].clone();
                        self.maker = Some(GoblinMaker::from(self.assets, data.1, Some(data.0)));
                    } else {
                        self.maker = Some(GoblinMaker::new(self.assets));
                    }
                }
                MenuUpdateResult::PlayOnline(level, name, author) => {
                    self.runtime =
                        Some(GoblinRuntime::new(self.assets, level, Some((name, author))))
                }
                _ => {}
            }
        }
        if DEBUG_ARGS.fps_counter {
            draw_text(&get_fps().to_string(), 64.0, 64.0, 32.0, WHITE);
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "goblin maker".to_string(),
        window_width: SCREEN_WIDTH as i32 * 2,
        window_height: SCREEN_HEIGHT as i32 * 2,
        platform: miniquad::conf::Platform {
            swap_interval: if DEBUG_ARGS.uncapped_fps {
                Some(0)
            } else {
                None
            },
            ..Default::default()
        },
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    println!("goblin maker v{VERSION}");
    let assets = Assets::default();
    let mut game = GameManager::new(&assets);
    loop {
        game.update();
        next_frame().await
    }
}
