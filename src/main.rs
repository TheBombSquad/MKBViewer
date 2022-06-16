#[macro_use]
extern crate num_derive; 

mod app;
mod stagedef;
mod parser;

fn main() {
    let app = app::Application::new();
    app.run();
}
