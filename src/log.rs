use std::fmt::Arguments;
use std::sync::Mutex;

use ratatui::style::Style;
use ratatui::text::Text;

pub static LOG: Mutex<Text<'static>> = Mutex::new(Text {
    lines: vec![],
    style: Style::new(),
    alignment: None,
});

pub fn log_format(args: Arguments<'_>) {
    if let Some(text) = args.as_str() {
        let mut log = LOG.lock().unwrap();
        for line in text.lines() {
            log.push_line(line.to_string());
        }
    } else {
        let text = args.to_string();
        let mut log = LOG.lock().unwrap();
        for line in text.lines() {
            log.push_line(line.to_string());
        }
    }
}

#[allow(unused)]
macro_rules! log {
    () => {
        log!("")
    };

    ($($arg:tt)*) => {{
        $crate::log::log_format(format_args!($($arg)*));
    }};
}

pub(crate) use log;
