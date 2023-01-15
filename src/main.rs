//! A viewer and editor for Monkey Ball stage files written in Rust that runs on native platforms
//! as well as on the web.
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate num_derive;

mod app;
mod parser;
mod renderer;
mod stagedef;

use tracing::Level;
/// Verbosity of console logs.
const LOG_LEVEL: Level = Level::DEBUG;

// Not web
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    //let log_config = tracing_subscriber::fmt::format().
    //
    tracing_subscriber::fmt().with_max_level(LOG_LEVEL).init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MKBViewer",
        native_options,
        Box::new(|cc| Box::new(app::MkbViewerApp::new(cc))),
    );
}

// Web
#[cfg(target_arch = "wasm32")]
use poll_promise::Promise;

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.

    console_error_panic_hook::set_once();

    let log_config = tracing_wasm::WASMLayerConfigBuilder::new().set_max_level(LOG_LEVEL).build();

    tracing_wasm::set_as_global_default_with_config(log_config);

    let web_options = eframe::WebOptions::default();

    let _start_web = Promise::spawn_async(async {
        eframe::start_web(
            "mkbviewer_canvas",
            web_options,
            Box::new(|cc| Box::new(app::MkbViewerApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
