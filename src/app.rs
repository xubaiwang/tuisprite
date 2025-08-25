use std::io;

use ratatui::{
    Terminal,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{WindowSize, window_size},
    },
    layout::Rect,
    prelude::Backend,
    style::{Color, Style},
    widgets::Widget,
};

use crate::{canvas::Canvas, drawing::Drawing, sgr_pixel::EnableSgrPixel};

/// 程序狀態。
pub struct App {
    should_quit: bool,
    s: String,
    /// 畫作內容。
    drawing: Drawing,
    window_size: Option<WindowSize>,
    canvas_area: Option<Rect>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            s: Default::default(),
            drawing: Default::default(),
            window_size: window_size().ok(),
            canvas_area: Default::default(),
        }
    }
}

impl App {
    /// 在給定終端上運行。
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) {
        // 啟用鼠標像素模式
        execute!(io::stdout(), EnableMouseCapture, EnableSgrPixel).unwrap();

        // 繪製-事件循環
        loop {
            terminal
                .draw(|frame| {
                    frame.render_widget(&mut self, frame.area());
                })
                .expect("failed to draw frame");

            self.read_and_handle_event();
            if self.should_quit {
                break;
            }
        }
        // 清理鼠標
        execute!(io::stdout(), DisableMouseCapture).unwrap();
    }

    /// 讀取並處理事件
    pub fn read_and_handle_event(&mut self) {
        let event = event::read().expect("failed to read event");
        // self.s = format!("{event:?}");

        match event {
            Event::Key(key_event) => {
                if key_event.code == KeyCode::Char('q') {
                    self.should_quit = true;
                }
            }
            Event::Mouse(mouse_event) => {
                if let Some((px, py)) = self.viewport_to_canvas(mouse_event.column, mouse_event.row)
                {
                    if let Some(pixel) = self.drawing.pixel_mut(px as usize, py as usize) {
                        self.s = format!("px = {px}, py = {py}");
                        *pixel = Color::Red;
                    }
                } else {
                    // self.s = "canvas cood is None".into();
                }
            }
            Event::Resize(_, _) => {
                self.window_size = window_size().ok();
            }
            _ => {}
        }
    }

    /// 視口轉畫布座標。
    pub fn viewport_to_canvas(&self, x: u16, y: u16) -> Option<(u16, u16)> {
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

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // 渲染畫布。
        Canvas::new(&self.drawing).render(area, buf);
        self.canvas_area = Some(area);

        // 渲染事件
        buf.set_string(0, area.bottom() - 1, &self.s, Style::default());
        // 渲染窗口大小
        buf.set_string(
            0,
            area.bottom() - 2,
            format!("{:?}", window_size()),
            Style::default(),
        );
    }
}
