use std::{fs, io, path::PathBuf};

use anyhow::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseButton,
            MouseEvent, MouseEventKind,
        },
        execute,
        terminal::{WindowSize, window_size},
    },
    layout::{Constraint, Layout, Rect},
};

use crate::{
    drawing::{Color, Drawing},
    sgr_pixel::EnableSgrPixel,
    widgets::{command_bar::CommandBar, status_bar::StatusBar, workspace::Workspace},
};

pub struct App {
    /// Whether the app should exit.
    should_exit: bool,
    path: Option<PathBuf>,
    /// The data of actual drawing.
    drawing: Drawing,

    color: Color,

    // Retained areas.
    window_size: Option<WindowSize>,
    canvas_area: Option<Rect>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            path: None,
            color: Color::RED,
            drawing: Default::default(),
            window_size: window_size().ok(),
            canvas_area: Default::default(),
        }
    }
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        let text = fs::read_to_string(&path)?;
        let mut drawing = serde_json::from_str::<Drawing>(&text)?;

        dbg!(drawing.validate());

        Ok(Self {
            should_exit: false,
            path: Some(path),
            color: Color::RED,
            drawing,
            window_size: window_size().ok(),
            canvas_area: None,
        })
    }

    /// Run the app loop.
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // validate the drawing
        if !self.drawing.validate() {
            panic!("drawing is invliad");
        }

        enable_mouse()?;

        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_event()?;
        }

        disable_mouse()?;

        Ok(())
    }

    /// We need to store retained state, so `&mut self` is used.
    fn render(&mut self, frame: &mut Frame) {
        // split layout into three
        let layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        frame.render_stateful_widget(
            Workspace::new(&self.drawing),
            layout[0],
            &mut self.canvas_area,
        );
        frame.render_widget(StatusBar, layout[1]);
        frame.render_widget(CommandBar, layout[2]);
    }

    fn handle_event(&mut self) -> Result<()> {
        let event = event::read().expect("failed to read event");

        match event {
            Event::Key(key) => self.on_key(key)?,
            Event::Mouse(mouse) => self.on_mouse(mouse),
            // NOTE: we need both cell size and pixel size, so the resize event fields is not used.
            Event::Resize(_, _) => self.on_resize(),
            _ => {}
        }

        Ok(())
    }

    /// Handle key event.
    fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.should_exit = true;
            }
            KeyCode::Char('w') => {
                self.write()?;
            }
            KeyCode::Char('1') => {
                self.color = Color::RED;
            }
            KeyCode::Char('2') => {
                self.color = Color::GREEN;
            }
            KeyCode::Char('3') => {
                self.color = Color::BLUE;
            }
            KeyCode::Char('4') => {
                self.color = Color::BLACK;
            }
            _ => {}
        }
        Ok(())
    }

    fn write(&self) -> Result<()> {
        if let Some(path) = &self.path {
            fs::write(path, serde_json::to_string(&self.drawing)?)?;
        }
        Ok(())
    }

    /// Handle mouse event.
    fn on_mouse(&mut self, mouse: MouseEvent) {
        if let Some((px, py)) = self.viewport_to_canvas(mouse.column, mouse.row) {
            if let Some(pixel) = self.drawing.pixel_mut(px as usize, py as usize) {
                match mouse.kind {
                    MouseEventKind::Down(mouse_button) | MouseEventKind::Drag(mouse_button) => {
                        match mouse_button {
                            MouseButton::Left => {
                                *pixel = self.color;
                            }
                            MouseButton::Right => {
                                *pixel = Color::RESET;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        } else {
            // self.s = "canvas cood is None".into();
        }
    }

    /// Handle resize event.
    fn on_resize(&mut self) {
        // NOTE: window_size return size in both cells and pixels
        self.window_size = crossterm::terminal::window_size().ok()
    }

    /// Transform viewport position to canvas position.
    ///
    /// Return `None` when position is outside canvas.
    fn viewport_to_canvas(&self, x: u16, y: u16) -> Option<(u16, u16)> {
        let window_size = self.window_size.as_ref()?;
        let canvas_area = self.canvas_area.as_ref()?;

        let cell_width = window_size.width / window_size.columns;
        let cell_height = window_size.height / window_size.rows;
        if !canvas_area.contains((x / cell_width, y / cell_height).into()) {
            return None;
        }
        let x_pixel = x / cell_width - canvas_area.x;
        let y_pixel = (y - canvas_area.y * cell_height) / (cell_height / 2);
        Some((x_pixel, y_pixel))
    }
}

/// Enable mouse and SGR Pixel mode.
fn enable_mouse() -> Result<()> {
    execute!(io::stdout(), EnableMouseCapture, EnableSgrPixel)?;
    Ok(())
}

/// Disable mouse and SGR Pixel mode.
fn disable_mouse() -> Result<()> {
    execute!(io::stdout(), DisableMouseCapture)?;
    Ok(())
}
