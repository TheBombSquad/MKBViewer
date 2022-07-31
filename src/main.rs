#[macro_use]
extern crate num_derive;

mod app;
mod parser;
mod stagedef;

fn main() {
    let app = app::Application::new();
    app.run();
}
