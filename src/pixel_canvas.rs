use ratatui::widgets::canvas::Context;

use crate::image::Sprite;
use crate::math::{Pos, Size};

pub struct PixelCanvas<'a, 'b> {
    pub ctx: &'a mut Context<'b>,
    pub origin: Pos<i32>,
    pub size: Size<u16>,
}

impl<'a, 'b> PixelCanvas<'a, 'b> {
    pub fn draw(&mut self, sprite: Sprite) {
        if let Some(rect) = sprite.rect(self.origin, self.size) {
            self.ctx.draw(&rect);
        }
    }
}
