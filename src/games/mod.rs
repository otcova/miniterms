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
}
