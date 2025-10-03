use std::{
    cell::RefCell,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use andromeda_core::{HostData, RuntimeHostHooks};
use andromeda_runtime::RuntimeMacroTask;
use anyhow::Result;
use csscolorparser::Color;
use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList, BuiltinFunctionArgs, create_builtin_function},
        execution::{
            Agent, JsResult,
            agent::{GcAgent, Options, RealmRoot},
        },
        scripts_and_modules::script::{parse_script, script_evaluation},
        types::{
            self, InternalMethods, IntoFunction, Object, PropertyDescriptor, PropertyKey, Value,
        },
    },
    engine::context::{Bindable, GcScope},
};
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
use unicode_width::UnicodeWidthStr;

use crate::{
    drawing::Drawing,
    sgr_pixel::EnableSgrPixel,
    widgets::{command_bar::CommandBar, status_bar::StatusBar, workspace::Workspace},
};

pub enum Mode {
    Normal,
    /// Colon and input command.
    Command(String),
}

impl Mode {
    fn as_command(&self) -> Option<&str> {
        match self {
            Mode::Normal => None,
            Mode::Command(command) => Some(command),
        }
    }
}

/// App runtime setting.
pub struct Setting {
    /// Current color.
    pub color: Color,
}

pub struct App {
    /// Whether the app should exit.
    should_exit: bool,
    path: Option<PathBuf>,
    /// The data of actual drawing.
    drawing: Drawing,

    /// Current mode.
    mode: Mode,

    // Retained areas.
    window_size: Option<WindowSize>,
    canvas_area: Option<Rect>,

    setting: Arc<RefCell<Setting>>,

    agent: GcAgent,
    realm: RealmRoot,
}

struct SettingResource {
    setting: Arc<RefCell<Setting>>,
}

fn set_color<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let color = args.get(0).to_string(agent, gc.reborrow()).unbind()?;

    let host_data = agent
        .get_host_data()
        .downcast_ref::<HostData<RuntimeMacroTask>>()
        .unwrap();
    let mut storage = host_data.storage.borrow_mut();
    let res = storage.get_mut::<SettingResource>().unwrap();

    if let Ok(color) = csscolorparser::Color::from_html(color.to_string_lossy(agent)) {
        res.setting.borrow_mut().color = color;
    };

    Ok(Value::Undefined)
}

fn prepare_js(setting: Arc<RefCell<Setting>>) -> (GcAgent, RealmRoot) {
    let (_macro_task_tx, _macro_task_rx) = mpsc::channel();
    let host_data = HostData::new(_macro_task_tx);

    {
        let mut map = host_data.storage.borrow_mut();
        map.insert(SettingResource { setting });
    }

    let host_hooks = RuntimeHostHooks::new(host_data);
    let host_hooks: &RuntimeHostHooks<RuntimeMacroTask> = &*Box::leak(Box::new(host_hooks));

    let mut agent = GcAgent::new(
        Options {
            disable_gc: false,
            print_internals: false,
        },
        host_hooks,
    );

    let create_global_object: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> = None;
    let create_global_this_value: Option<for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>> =
        None;
    let realm = agent.create_realm(
        create_global_object,
        create_global_this_value,
        Some(
            |agent: &mut Agent, global_object: Object<'_>, mut gc: GcScope<'_, '_>| {
                // builtin
                let function = create_builtin_function(
                    agent,
                    nova_vm::ecmascript::builtins::Behaviour::Regular(set_color),
                    BuiltinFunctionArgs::new(1, "_set_color"),
                    gc.nogc(),
                );

                let property_key = PropertyKey::from_static_str(agent, "color", gc.nogc());
                global_object
                    .internal_define_own_property(
                        agent,
                        property_key.unbind(),
                        PropertyDescriptor {
                            set: Some(Some(function.into_function().unbind())),
                            ..Default::default()
                        },
                        gc.reborrow(),
                    )
                    .unwrap();
            },
        ),
    );

    (agent, realm)
}

impl Default for App {
    fn default() -> Self {
        let setting = Arc::new(RefCell::new(Setting {
            color: Color::from_rgba8(0, 0, 0, 255),
        }));
        let (agent, realm) = prepare_js(setting.clone());
        Self {
            mode: Mode::Normal,
            should_exit: false,
            path: None,
            drawing: Default::default(),
            window_size: window_size().ok(),
            canvas_area: Default::default(),
            setting,
            agent,
            realm,
        }
    }
}

fn load_drawing_from_file(path: &Path) -> Result<Drawing> {
    let text = fs::read_to_string(path)?;
    let mut drawing = serde_json::from_str::<Drawing>(&text)?;
    drawing.validate();
    Ok(drawing)
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        let drawing = load_drawing_from_file(&path).unwrap_or_default();

        let setting = Arc::new(RefCell::new(Setting {
            color: Color::from_rgba8(0, 0, 0, 255),
        }));
        let (agent, realm) = prepare_js(setting.clone());

        Ok(Self {
            should_exit: false,
            mode: Mode::Normal,
            path: Some(path),
            drawing,
            window_size: window_size().ok(),
            canvas_area: None,
            setting,
            agent,
            realm,
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
        frame.render_widget(StatusBar::new(&self.setting.borrow()), layout[1]);
        frame.render_widget(CommandBar::new(self.mode.as_command()), layout[2]);

        match &self.mode {
            Mode::Command(command) => {
                frame.set_cursor_position((1 + command.width() as u16, frame.area().bottom() - 1));
            }
            _ => {}
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
        match &mut self.mode {
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
                        self.mode = Mode::Command(String::new());
                    }
                    KeyCode::Char('+') => {
                        self.drawing
                            .resize(self.drawing.width + 1, self.drawing.height + 1);
                    }
                    KeyCode::Char('-') => {
                        if self.drawing.width > 1 {
                            self.drawing
                                .resize(self.drawing.width - 1, self.drawing.height - 1);
                        }
                    }
                    _ => {}
                }
            }
            Mode::Command(command) => match key.code {
                KeyCode::Backspace => {
                    command.pop();
                }
                KeyCode::Enter => {
                    self.execute_command();
                }
                KeyCode::Char(ch) => {
                    command.push(ch);
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                }
                _ => {}
            },
        }
        Ok(())
    }

    fn execute_command(&mut self) {
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        let Mode::Command(command) = mode else {
            panic!("not command mode")
        };

        self.agent.run_in_realm(&self.realm, |agent, mut gc| {
            let realm_obj = agent.current_realm(gc.nogc());
            let source_text = types::String::from_str(agent, &command, gc.nogc());
            let script = match parse_script(agent, source_text, realm_obj, true, None, gc.nogc()) {
                Ok(script) => script,
                _ => panic!("invalid script"),
            };

            let result = script_evaluation(agent, script.unbind(), gc.reborrow()).unbind();
            match result {
                Ok(result) => match result.to_string(agent, gc) {
                    Ok(_val) => {
                        // println!("{}", val.to_string_lossy(agent));
                    }
                    Err(_) => {
                        // eprintln!("error converting result to string")
                    }
                },
                Err(error) => {
                    let _error_value = error.value();
                    // let error_message = error_value
                    //     .string_repr(agent, gc.reborrow())
                    //     .as_str(agent)
                    //     .unwrap()
                    //     .to_string();
                    // eprintln!("{}", error_message);
                }
            }
        });
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
                                *pixel = self.setting.borrow().color.clone();
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
