#[macro_use]
extern crate num_derive;

mod app;
mod parser;
mod stagedef;

// Web
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MKBViewer",
        native_options,
        Box::new(|cc| Box::new(app::MkbViewerApp::new(cc))),
    );
}

// Not web
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "mkbviewer_canvas",
        web_options,
        Box::new(|cc| Box::new(app::MkbViewerApp::new(cc))),
    )
    .expect("failed to start eframe");
}

