use macroquad::prelude::*;

use crate::{
    assets::Assets,
    maker::*,
    menu::{MainMenu, MenuUpdateResult},
    runtime::*,
    utils::*,
};

mod assets;
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
    menu: MainMenu<'a>,
    runtime: Option<GoblinRuntime<'a>>,
    maker: Option<GoblinMaker<'a>>,
}

impl<'a> GameManager<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self {
            menu: MainMenu::new(assets),
            maker: None,
            assets,
            runtime: None,
        }
    }
    fn update(&mut self) {
        if is_key_pressed(KeyCode::E) && self.runtime.is_some() {
            self.runtime = None;
        }
        if let Some(runtime) = &mut self.runtime {
            runtime.update();
        } else if let Some(maker) = &mut self.maker {
            let result = maker.update();
            if result {
                self.runtime = Some(GoblinRuntime::new(self.assets, maker.level.clone()));
            }
        } else {
            // neither runtime or maker is open, draw main menu
            let result = self.menu.update();
            if let MenuUpdateResult::Create = result {
                self.maker = Some(GoblinMaker::new(self.assets));
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
