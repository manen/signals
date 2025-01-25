# `signals`

silly signal game

## try out for yourself

release builds for windows and linux are compiled on every commit using github actions. \
[see actions](https://github.com/manen/signals/actions/)

## todo

see [roadmap](/roadmap.md)

## building for the web

you'll need [run](https://github.com/manen/run) for this, and the `wasm32-unknown-emscripten` build target. \
you can install them using:

```sh
cargo install --git https://github.com/manen/run
rustup target add wasm32-unknown-emscripten
```
