use macroquad::prelude::*;

use crate::{assets::Assets, maker::*, runtime::*, utils::*};

mod assets;
mod level;
mod maker;
mod player;
mod runtime;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn window_conf() -> Conf {
    Conf {
        window_title: "goblin maker".to_string(),
        window_width: SCREEN_WIDTH as i32 * 2,
        window_height: SCREEN_HEIGHT as i32 * 2,
        platform: miniquad::conf::Platform {
            //swap_interval: Some(0),
            ..Default::default()
        },
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    println!("goblin maker v{VERSION}");
    let assets = Assets::default();
    let mut game = GoblinMaker::new(&assets);
    loop {
        game.update();
        next_frame().await
    }
}
