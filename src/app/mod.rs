use std::{cell::RefCell, path::PathBuf, pin::Pin, rc::Rc};

use anyhow::Result;
use crossterm::{
    self,
    event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    terminal::{WindowSize, window_size},
};
use csscolorparser::Color;
use itertools::Itertools;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
};
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::{Stream, StreamExt};

pub mod action;
pub mod config;
pub mod runtime;

use crate::{
    app::{
        action::Action,
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

    config: Rc<RefCell<Config>>,

    runtime: RefCell<Runtime>,

    tx: UnboundedSender<Event>,

    stream: Pin<Box<dyn Stream<Item = Event>>>,

    message: Option<String>,
}

impl App {
    /// Create a new app.
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let config = Rc::new(RefCell::new(Config::default()));
        let runtime = RefCell::new(Runtime::new(config.clone()));

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let crossterm_stream = ::crossterm::event::EventStream::new()
            // .timeout(Duration::from_millis(1000))
            .filter_map(|e| match e {
                Ok(event) => Some(Event::Terminal(event)),
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
                        crossterm::event::Event::Key(key) => self.on_key(key)?,
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
    fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        let drawing = self.drawing.as_mut().unwrap();
        let action = match &self.config.borrow().mode {
            Mode::Normal => match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('w') => Action::Save(None),
                KeyCode::Char(':') => Action::EnterCommandMode,
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    Action::Resize(drawing.width + 1, drawing.height + 1)
                }
                KeyCode::Char('-') => {
                    if drawing.width > 1 {
                        Action::Resize(drawing.width - 1, drawing.height - 1)
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Char('E') => Action::Erase,
                KeyCode::Char(ch @ '1')
                | KeyCode::Char(ch @ '2')
                | KeyCode::Char(ch @ '3')
                | KeyCode::Char(ch @ '4')
                | KeyCode::Char(ch @ '5')
                | KeyCode::Char(ch @ '6')
                | KeyCode::Char(ch @ '7')
                | KeyCode::Char(ch @ '8')
                | KeyCode::Char(ch @ '9') => {
                    let index = ch.to_digit(10).unwrap() as u8;
                    Action::SetColor(either::Either::Right(index))
                }
                _ => return Ok(()),
            },
            Mode::Command(command) => match key.code {
                KeyCode::Esc => Action::EnterNormalMode,
                KeyCode::Char(ch) => Action::CommandPush(ch),
                KeyCode::Backspace => Action::CommandPop,
                KeyCode::Enter => Action::Execute(command.clone()),
                _ => return Ok(()),
            },
        };
        self.perform(action)?;
        Ok(())
    }

    fn perform(&mut self, action: Action) -> Result<()> {
        // NOTE: borrow_mut must be called in each individual branch,
        // as execute_script also borrow mutably.
        match action {
            Action::Quit => self.should_exit = true,
            Action::Save(path) => self.write(path)?,
            Action::EnterCommandMode => {
                self.config.borrow_mut().mode = Mode::Command(String::new())
            }
            Action::EnterNormalMode => self.config.borrow_mut().mode = Mode::Normal,
            Action::CommandPush(ch) => match &mut self.config.borrow_mut().mode {
                Mode::Normal => self
                    .tx
                    .send(Event::Message("Not command mode".to_string()))?,
                Mode::Command(command) => command.push(ch),
            },
            Action::CommandPop => match &mut self.config.borrow_mut().mode {
                Mode::Normal => self
                    .tx
                    .send(Event::Message("Not command mode".to_string()))?,
                Mode::Command(command) => {
                    command.pop();
                }
            },
            Action::Resize(w, h) => {
                if let Some(drawing) = self.drawing.as_mut() {
                    drawing.resize(w, h);
                } else {
                    self.tx
                        .send(Event::Message("drawing is None".to_string()))?
                }
            }
            Action::Erase => {
                if let Some(drawing) = self.drawing.as_mut() {
                    drawing.erase_all();
                } else {
                    self.tx
                        .send(Event::Message("drawing is None".to_string()))?
                }
            }
            Action::SetColor(either) => match either {
                either::Either::Left(color) => {
                    self.config.borrow_mut().set_color(color);
                }
                either::Either::Right(index) => {
                    let mut config = self.config.borrow_mut();
                    let color = config
                        .color_history
                        .iter()
                        .nth_back(index as usize)
                        .cloned();
                    if let Some(color) = color {
                        config.set_color(color);
                    }
                }
            },
            Action::Execute(command) => {
                match command.strip_prefix('=') {
                    Some(script) => {
                        self.runtime.borrow_mut().execute_script(script)?;
                    }
                    None => {
                        self.run_command(&command)?;
                    }
                }
                self.config.borrow_mut().mode = Mode::Normal;
            }
        }

        Ok(())
    }

    fn run_command(&mut self, command: &str) -> Result<()> {
        // TODO: should use something like shlex or vim syntax parser
        let command = command.split_ascii_whitespace().collect_vec();
        match command.as_slice() {
            ["w"] => self.perform(Action::Save(None))?,
            ["w", path] => self.perform(Action::Save(Some(PathBuf::from(path))))?,
            ["q"] => {
                self.perform(Action::Quit)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn write(&self, path: Option<PathBuf>) -> Result<()> {
        let tx = self.tx.clone();
        // TODO: make drawing arc
        let serialized = serde_json::to_string(&self.drawing)?;
        if let Some(path) = path.or(self.path.to_owned()) {
            // TODO: make path arc
            let path = path.to_path_buf();
            tokio::spawn(async move {
                tokio::fs::write(path, serialized).await.unwrap();
                tx.send(Event::Message("write success".to_string()))
                    .unwrap();
            });
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
                                *pixel = self.config.borrow().color.clone();
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
