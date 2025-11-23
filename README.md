# Goblin Maker

<img width="1023" height="576" alt="Banner" src="https://github.com/user-attachments/assets/18c0860a-7bee-421c-b32e-a046d4ed4cfd" />


A Super Mario Maker inspired game where players create and share their own levels!

The game comes not only with a built-in level editor, but also a level browser where you can see and play other user's levels! I used hackclub's [Nest](https://hackclub.app/) for hosting the server which your levels are uploaded to. 

The game itself was written entirely in Rust, and the server was made with Flask.

When creating levels, you need the player placed somewhere, as well as the finish flag. To publish a level, you also need to complete it once without making any changes, to verify that it's possible.

<img width="1021" height="575" alt="image" src="https://github.com/user-attachments/assets/51f32b34-fb0c-42c7-b74d-f0ba9547894c" />
<sup>Browse online levels made by other players!</sup>

## About

This project was made for hackclub's siege, with the theme *Framework*. Entire project was done in just a week, and I'm really proud of it! It turned out really well I think.

## Controls

In runtime you move with WASD and jump with Space.

In editor you can select different tools, either by pressing their icon in the top left corner, or with their keybind:
- B: Pencil tool
- E: Eraser tool
- S: Shape tool

In the editor you can test your level by pressing the play button at the top center of the screen, or by pressing R.

## Building

Since the project is made in Rust, you'll need that installed.

To run natively, you should be able to do
```bash
cargo run
```

To build for web, served with for example `basic-http-server`, you'll do:
```bash
cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/goblin-maker.wasm web/ && basic-http-server web/
```
