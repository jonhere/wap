# wap
[![Build Status](https://travis-ci.org/jonhere/wap.svg?branch=master)](https://travis-ci.org/jonhere/wap)
[![Build status](https://ci.appveyor.com/api/projects/status/github/jonhere/wap?svg=true)](https://ci.appveyor.com/project/jonhere/wap)

Wap library allows you to write a web page (or node.js) app exclusively in Rust.
All you need to start is the boilerplate wap.js and html configured with title and link to your .wasm file.

Target is exclusively for Rusts wasm32-unknown-unknown. For cross-platform project probably better going with wasm32-unknown-emscripten.

Wap functions give the wasm low level calls into JavaScript environment. It does not directly provide a higher level library for easy access to API (DOM); one could be created on top.

## Note
* Version 0.1.x is very much unstable work in progress.
* [release notes](https://github.com/jonhere/wap/releases)
* Alot of javascript iregularity so maybe major changes.
* Direction unknown, not aimed as creating highest of speed code but good as starting point. Will be kept minimal.
* Reading the source highly advised.
* You can always call eval.

## Usage
* rustup target add wasm32-unknown-unknown
* cargo new --bin NAME
* edit Cargo.toml
* - [dependencies]
* - wap = { git = "https://github.com/jonhere/wap" }
* copy then edit [hello_world_release.html](https://raw.githubusercontent.com/jonhere/wap/master/hello_world_release.html) as NAME_release.html to project root
* - title
* - hello_world.wasm to NAME.wasm
* copy [hello_world.rs](https://raw.githubusercontent.com/jonhere/wap/master/examples/hello_world.rs) to src/main.rs
* add .cargo/config
* - [build]
* - target = "wasm32-unknown-unknown"
* cargo build --release
* Open in firefox
* Chrome requires webserver (e.g. cargo install simple-http-server )

#### node.js
* node -e 'require("./target/wasm32-unknown-unknown/release/wap.js").wap("target/wasm32-unknown-unknown/release/examples/hello_node.wasm")'

## License
Apache 2.0 or MIT.
wap.js public domain
