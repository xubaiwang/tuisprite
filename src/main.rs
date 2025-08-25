use crate::app::App;

mod app;
mod canvas;
mod drawing;
mod sgr_pixel;

fn main() {
    let mut terminal = ratatui::init();
    App::default().run(&mut terminal);
    ratatui::restore();
}
