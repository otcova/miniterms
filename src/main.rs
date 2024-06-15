mod games;
mod image;
mod input;
mod math;
mod pixel_canvas;
mod solution;

use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use games::{trex::TRexGame, GameContext};
use input::Keys;
use math::{Pos, Size};
use pixel_canvas::PixelCanvas;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::Color,
    symbols::Marker,
    terminal::{Frame, Terminal},
    text::Text,
    widgets::{
        canvas::{Canvas, Map, MapResolution},
        Block, Paragraph, Widget,
    },
};
use solution::Solution;

fn main() -> io::Result<()> {
    App::run()
}

struct App {
    trex: TRexGame,
    keys: Keys,
    solution: Solution,
    close: bool,
    log: Text<'static>,
}

impl App {
    fn new() -> Self {
        Self {
            trex: TRexGame::new(),
            keys: Keys::new(),
            solution: Solution::new(),
            close: false,
            log: Text::default(),
        }
    }

    pub fn run() -> io::Result<()> {
        let mut terminal = init_terminal()?;
        let mut app = Self::new();

        // if less than `tick_margin` time is left, do not sleep, insted do a busy wait.
        let tick_margin = Duration::from_millis(5);
        let tick_rate = Duration::from_millis(40); // 25 fps
        let mut last_tick = Instant::now();

        while !app.close {
            let timeout = tick_rate.saturating_sub(last_tick.elapsed() + tick_margin);

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    app.handle_key(key);
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick += tick_rate;
                let _ = terminal.draw(|frame| app.ui(frame));
            }
        }

        restore_terminal()
    }

    fn handle_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.close = true,
            KeyCode::Esc => self.close = true,
            _ => self.keys.handle_key_event(key_event),
        }
    }

    fn ui(&mut self, frame: &mut Frame) {
        use Constraint::*;

        let log_width = if self.log.height() == 0 { 0 } else { 50 };

        let horizontal = Layout::horizontal([Length(log_width), Length(22), Fill(1), Fill(3)]);
        let [log_column, column_a, column_b, column_c] = horizontal.areas(frame.size());

        let column_b_layout = Layout::vertical([Length(column_b.width / 2), Fill(1)]);
        let [rect_b_a, rect_b_b] = column_b_layout.areas(column_b);

        let column_c_layout = Layout::vertical([Fill(1), Fill(1), Fill(1)]);
        let [rect_c_a, rect_c_b, rect_c_c] = column_c_layout.areas(column_c);

        if !log_width != 0 {
            frame.render_widget(self.log_widget(log_column.as_size().into()), log_column);
        }

        frame.render_widget(self.frame("Tetris"), column_a);
        frame.render_widget(self.frame("Defend the Planet"), rect_b_a);
        frame.render_widget(self.frame("Breakout"), rect_b_b);
        frame.render_widget(self.trex_canvas(rect_c_a.as_size().into()), rect_c_a);
        frame.render_widget(self.frame("Space"), rect_c_b);
        frame.render_widget(self.frame("Packman"), rect_c_c);

        self.keys.update();
        self.solution.update();
    }

    fn log_widget(&self, area: Size<u16>) -> impl Widget + '_ {
        let scroll = (self.log.height() & !15).saturating_sub(area.height as usize + 17) as u16;

        Paragraph::new(self.log.clone())
            .block(Block::bordered().title("Log"))
            .scroll((scroll, 0))
    }

    fn frame(&self, title: &'static str) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title(title))
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Green,
                    resolution: MapResolution::High,
                });
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
    }

    fn trex_canvas(&mut self, canvas_size: Size<u16>) -> impl Widget + '_ {
        let size = Size::new(2 * (canvas_size.width - 2), 4 * (canvas_size.height - 2));

        {
            let mut log = String::new();

            self.trex.update(&mut GameContext {
                size,
                keys: self.keys,
                solution: &self.solution,
                log: &mut log,
            });

            for line in log.lines() {
                self.log.push_line(line.to_string());
            }
        }

        Canvas::default()
            .block(Block::bordered().title("T-Rex"))
            .marker(Marker::Braille)
            .paint(move |ctx| {
                self.trex.draw(&mut PixelCanvas {
                    ctx,
                    size,
                    origin: Pos::new(20, size.height as i32 - 1),
                });
            })
            .x_bounds([0., 1.])
            .y_bounds([0., 1.])
    }
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
