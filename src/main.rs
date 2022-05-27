use fltk::{prelude::*, *};

mod app;
mod stagedef;

fn main() {
    let app = app::Application::new();
    app.run();
}
