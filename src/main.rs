use crate::app::App;

mod app;
mod canvas;
mod drawing;
mod sgr_pixel;

fn main() {
    let mut terminal = ratatui::init();

    let path = std::env::args().nth(1);

    let app = match path {
        Some(path) => App::from_path(path),
        None => App::default(),
    };

    app.run(&mut terminal);
    ratatui::restore();
}
