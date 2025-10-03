use std::{cell::RefCell, fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use csscolorparser::Color;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{self, Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
        terminal::{WindowSize, window_size},
    },
    layout::{Constraint, Layout, Rect},
};

pub mod config;
pub mod runtime;

use crate::{
    app::{
        config::{Config, mode::Mode},
        runtime::Runtime,
    },
    drawing::{Drawing, io::load_drawing_from_file},
    utils::mouse::{disable_mouse, enable_mouse},
    widgets::{command_bar::CommandBar, status_bar::StatusBar, workspace::Workspace},
};

pub struct App {
    /// Whether the app should exit.
    should_exit: bool,
    path: Option<PathBuf>,
    /// The data of actual drawing.
    drawing: Drawing,

    // Retained areas.
    window_size: Option<WindowSize>,
    canvas_area: Option<Rect>,

    config: Arc<RefCell<Config>>,

    runtime: RefCell<Runtime>,
}

impl Default for App {
    fn default() -> Self {
        let config = Arc::new(RefCell::new(Config::default()));
        let runtime = RefCell::new(Runtime::new(config.clone()));

        Self {
            should_exit: false,
            path: None,
            drawing: Default::default(),
            window_size: window_size().ok(),
            canvas_area: Default::default(),
            config,
            runtime,
        }
    }
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        let drawing = load_drawing_from_file(&path).unwrap_or_default();
        let config = Arc::new(RefCell::new(Config::default()));
        let runtime = RefCell::new(Runtime::new(config.clone()));

        Ok(Self {
            should_exit: false,
            path: Some(path),
            drawing,
            window_size: window_size().ok(),
            canvas_area: None,
            config,
            runtime,
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
        frame.render_widget(StatusBar::new(&self.config.borrow()), layout[1]);
        let mut position = None;
        frame.render_stateful_widget(
            CommandBar::new(&self.config.borrow()),
            layout[2],
            &mut position,
        );

        // Sync cursor position
        if let Some(position) = position {
            frame.set_cursor_position(position);
        }
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
        let config = self.config.borrow();
        let mode = &mut *config.mode.borrow_mut();
        match mode {
            Mode::Normal => {
                match key.code {
                    KeyCode::Char('q') => {
                        self.should_exit = true;
                    }
                    KeyCode::Char('w') => {
                        self.write()?;
                    }
                    KeyCode::Char(':') => {
                        // enter command mode
                        *mode = Mode::Command(Default::default());
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        self.drawing
                            .resize(self.drawing.width + 1, self.drawing.height + 1);
                    }
                    KeyCode::Char('-') => {
                        if self.drawing.width > 1 {
                            self.drawing
                                .resize(self.drawing.width - 1, self.drawing.height - 1);
                        }
                    }
                    KeyCode::Char('E') => {
                        self.drawing.erase_all();
                    }
                    KeyCode::Char(ch @ '1')
                    | KeyCode::Char(ch @ '2')
                    | KeyCode::Char(ch @ '3')
                    | KeyCode::Char(ch @ '4')
                    | KeyCode::Char(ch @ '5')
                    | KeyCode::Char(ch @ '6')
                    | KeyCode::Char(ch @ '7')
                    | KeyCode::Char(ch @ '8')
                    | KeyCode::Char(ch @ '9') => {
                        let color = ch.to_digit(10).and_then(|n| {
                            config
                                .color_history
                                .borrow()
                                .iter()
                                // n - 1 except 0 => 9
                                .nth_back((n as usize).checked_sub(1).unwrap_or(9))
                                .cloned()
                        });
                        if let Some(color) = color {
                            config.set_color(color);
                        }
                    }
                    // TODO: number back
                    _ => {}
                }
            }

            Mode::Command(command) => match key.code {
                KeyCode::Backspace => {
                    command.borrow_mut().pop();
                }
                KeyCode::Enter => {
                    self.runtime
                        .borrow_mut()
                        .execute_script(&command.borrow())?;
                    *mode = Mode::Normal;
                }
                KeyCode::Char(ch) => {
                    command.borrow_mut().push(ch);
                }
                KeyCode::Esc => {
                    *mode = Mode::Normal;
                }
                _ => {}
            },
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
                                *pixel = self.config.borrow().color.borrow().clone();
                            }
                            MouseButton::Right => {
                                *pixel = Color::from_rgba8(0, 0, 0, 0);
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
