use crate::input::Keys;
use crate::math::Size;
use crate::solution::Solution;
use std::fmt::Write;

pub mod tetris;
pub mod trex;

mod utils;

pub struct GameContext<'a> {
    pub size: Size<u16>,
    pub keys: Keys,
    pub solution: &'a Solution,
    pub log: &'a mut String,
}

impl<'a> Write for GameContext<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.log.write_str(s)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.log.write_fmt(args)
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.log.write_char(c)
    }
}
