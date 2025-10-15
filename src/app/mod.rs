use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    self,
    event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    terminal::{WindowSize, window_size},
};
use csscolorparser::Color;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
};
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::{Stream, StreamExt};

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

#[derive(Debug)]
enum Event {
    /// Wrap crossterm event.
    Terminal(::crossterm::event::Event),
    /// Report error message.
    Message(String),
}

pub struct App {
    /// Whether the app should exit.
    should_exit: bool,
    path: Option<PathBuf>,
    /// The data of actual drawing.
    drawing: Option<Drawing>,

    // Retained areas.
    window_size: Option<WindowSize>,
    canvas_area: Option<Rect>,

    config: Arc<RefCell<Config>>,

    runtime: RefCell<Runtime>,

    tx: UnboundedSender<Event>,

    stream: Pin<Box<dyn Stream<Item = Event>>>,

    message: Option<String>,
}

impl App {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let config = Arc::new(RefCell::new(Config::default()));
        let runtime = RefCell::new(Runtime::new(config.clone()));

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let crossterm_stream = ::crossterm::event::EventStream::new()
            .timeout(Duration::from_millis(1000))
            .filter_map(|e| match e {
                Ok(Ok(event)) => Some(Event::Terminal(event)),
                _ => None,
            });
        let rx_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

        let stream = Box::pin(crossterm_stream.merge(rx_stream));

        Ok(Self {
            drawing: None,
            should_exit: false,
            path,
            window_size: window_size().ok(),
            canvas_area: None,
            config,
            runtime,
            tx,
            message: None,
            stream,
        })
    }

    /// Run the app loop.
    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.drawing = Some(match &self.path {
            Some(path) => load_drawing_from_file(path).await.unwrap_or_default(),
            None => Drawing::default(),
        });
        // validate the drawing
        if !self.drawing.as_mut().unwrap().validate() {
            panic!("drawing is invliad");
        }

        enable_mouse()?;

        while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_event().await?;
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
            Workspace::new(&self.config.borrow(), self.drawing.as_ref().unwrap()),
            layout[0],
            &mut self.canvas_area,
        );
        frame.render_widget(StatusBar::new(&self.config.borrow()), layout[1]);
        let mut position = None;
        frame.render_stateful_widget(
            CommandBar::new(&self.config.borrow(), self.message.as_deref()),
            layout[2],
            &mut position,
        );

        // Sync cursor position
        if let Some(position) = position {
            frame.set_cursor_position(position);
        }
    }

    async fn handle_event(&mut self) -> Result<()> {
        if let Some(event) = self.stream.next().await {
            match event {
                Event::Terminal(event) => {
                    match event {
                        crossterm::event::Event::Key(key) => self.on_key(key).await?,
                        crossterm::event::Event::Mouse(mouse) => self.on_mouse(mouse),
                        // NOTE: we need both cell size and pixel size, so the resize event fields is not used.
                        crossterm::event::Event::Resize(_, _) => self.on_resize(),
                        _ => {}
                    }
                }
                Event::Message(message) => self.message = Some(message),
            }
        }

        Ok(())
    }

    /// Handle key event.
    async fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        let drawing = self.drawing.as_mut().unwrap();
        let config = self.config.borrow();
        let mode = &mut *config.mode.borrow_mut();
        match mode {
            Mode::Normal => {
                match key.code {
                    KeyCode::Char('q') => {
                        self.should_exit = true;
                    }
                    KeyCode::Char('w') => {
                        self.write(None).await?;
                    }
                    KeyCode::Char(':') => {
                        // enter command mode
                        *mode = Mode::Command(Default::default());
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        drawing.resize(drawing.width + 1, drawing.height + 1);
                    }
                    KeyCode::Char('-') => {
                        if drawing.width > 1 {
                            drawing.resize(drawing.width - 1, drawing.height - 1);
                        }
                    }
                    KeyCode::Char('E') => {
                        drawing.erase_all();
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

    async fn write(&self, path: Option<&Path>) -> Result<()> {
        if let Some(path) = path.or(self.path.as_deref()) {
            tokio::fs::write(path, serde_json::to_string(&self.drawing)?).await?;
            self.tx.send(Event::Message("write success".to_string()))?;
        } else {
            self.tx
                .send(Event::Message("no path specified".to_string()))?;
        }
        Ok(())
    }

    /// Handle mouse event.
    fn on_mouse(&mut self, mouse: MouseEvent) {
        if let Some((px, py)) = self.viewport_to_canvas(mouse.column, mouse.row) {
            if let Some(pixel) = self
                .drawing
                .as_mut()
                .unwrap()
                .pixel_mut(px as usize, py as usize)
            {
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
