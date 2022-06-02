mod app;
mod stagedef;
mod parser;

fn main() {
    let app = app::Application::new();
    app.run();
}
