use std::sync::LazyLock;

use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};

pub const SCREEN_WIDTH: f32 = 256.0 * 2.0;
pub const SCREEN_HEIGHT: f32 = 144.0 * 2.0;

pub const MAX_VELOCITY: f32 = 190.0;
pub const GROUND_FRICTION: f32 = 0.17 * 60.0;
pub const AIR_DRAG: f32 = 0.07 * 60.0;
pub const GRAVITY: f32 = 0.35 * 3600.0;
pub const ACCELERATION: f32 = 3600.0 / 2.0;

pub const SCROLL_AMT: f32 = 1.1;
pub const MIN_ZOOM: f32 = 0.001;

pub const SKY_COLOR: Color = Color::from_hex(0x29adff);
pub const MAKER_BG_COLOR: Color = Color::from_hex(0x365987);

#[derive(Default)]
pub struct DebugArgs {
    pub speedometer: bool,
    pub fps_counter: bool,
    pub uncapped_fps: bool,
}

pub static DEBUG_ARGS: LazyLock<DebugArgs> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    {
        let args: Vec<String> = std::env::args().collect();
        DebugArgs {
            speedometer: args.contains(&"spd".to_string()),
            fps_counter: args.contains(&"fps".to_string()),
            uncapped_fps: args.contains(&"uncap".to_string()),
        }
    }
    #[cfg(not(debug_assertions))]
    {
        DebugArgs::default()
    }
});

pub fn create_camera(w: f32, h: f32) -> Camera2D {
    let rt = render_target(w as u32, h as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    Camera2D {
        render_target: Some(rt),
        zoom: Vec2::new(1.0 / w * 2.0, 1.0 / h * 2.0),
        ..Default::default()
    }
}
pub fn get_input_axis() -> Vec2 {
    let mut i = Vec2::ZERO;
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        i.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        i.x += 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        i.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        i.y += 1.0;
    }
    i
}

pub static GRID_MATERIAL: LazyLock<Material> = LazyLock::new(|| {
    // to enable transparency!
    let pipeline = PipelineParams {
        alpha_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    let m = load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX_SHADER,
            fragment: GRID_FRAGMENT,
        },
        MaterialParams {
            pipeline_params: pipeline,
            uniforms: vec![
                UniformDesc::new("zoom", UniformType::Float1),
                UniformDesc::new("scale", UniformType::Float1),
                UniformDesc::new("offset", UniformType::Float2),
                UniformDesc::new("screen", UniformType::Float2),
            ],
            ..Default::default()
        },
    )
    .unwrap();
    m.set_uniform("zoom", 1.0_f32);
    m.set_uniform("offset", vec2(0.0, 0.0));
    m.set_uniform("screen", vec2(0.0, 0.0));
    m
});

pub const GRID_FRAGMENT: &str = "#version 100
precision mediump float;

uniform lowp float zoom;
uniform lowp float scale;
uniform lowp vec2 offset;
uniform lowp vec2 screen;

void main() {
    vec2 pos = gl_FragCoord.xy;
    float grid_size = 16.0*scale;
    vec4 color = vec4(0.0,0.0,0.0,0.0);
    vec4 border_color = vec4(0.25,0.25,0.25,0.25);
    float width = 1.0;
    float aspect = screen.x / screen.y;

    // all these if's do the same thing but im trying to find the one that is the least jittery

    //if (mod((((screen.y-pos.y) /zoom+ offset.y*scale)),grid_size) < width || mod((pos.x /zoom+ offset.x*scale),grid_size) < width) {
    //    color = border_color;
    //}
    //if (mod(floor(screen.y-pos.y + offset.y*scale*zoom),grid_size*zoom) < width || mod(floor(pos.x + offset.x*scale*zoom),grid_size*zoom) < width) {
    //    color = border_color;
    //}
    if (mod((((screen.y-pos.y) /zoom/scale+ offset.y)),grid_size/scale) < width || mod((pos.x /zoom/scale+ offset.x),grid_size/scale) < width) {
        color = border_color;
    }

    gl_FragColor = color;
}
";

pub const DEFAULT_VERTEX_SHADER: &str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";
