use crate::math::{Line, Pos, Rect, Size};
use ratatui::style::Color;
use ratatui::widgets::canvas::Shape;
use std::ops::Range;

#[derive(Clone, Copy)]
pub struct Image {
    pub pixels: &'static [u32],
    pub width: u16,
    pub color: Color,
}

pub struct ImageAnimation(pub &'static [Image]);

impl ImageAnimation {
    pub fn image(&self, frame: usize) -> Image {
        self.0[frame % self.0.len()]
    }
}

pub struct Sprite {
    pub image: Image,
    pub position: Pos<i32>,
    pub origin: Pos<Origin>,
}

pub struct SpriteRect {
    pub image: Image,
    pub image_offset: Pos<u16>,
    pub rect: Rect<u16>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Origin {
    Min,
    Max,
}

struct Clip {
    pub image_offset: u16,
    pub position: Line<u16>,
}

impl Clip {
    fn new(window_size: u16, range: Line<i32>) -> Option<Clip> {
        if range.end <= 0 || window_size as i32 <= range.start {
            return None;
        }

        Some(Clip {
            image_offset: (-range.start).max(0) as u16,
            position: Line {
                start: range.start.max(0) as u16,
                end: range.end.min(window_size as i32) as u16,
            },
        })
    }
}

impl Shape for SpriteRect {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        let mut image_y = self.image_offset.y as usize;

        for y in self.rect.y.range() {
            let mut bitmap = self.image.pixels[image_y] >> self.image_offset.x;

            for x in self.rect.x.range() {
                if bitmap & 1 == 1 {
                    painter.paint(x as usize, y as usize, self.image.color);
                }
                bitmap >>= 1;
            }

            image_y += 1;
        }
    }
}

impl Sprite {
    fn bounding_box(&self) -> Rect<i32> {
        let Pos { x, y } = self.position;

        let width = self.image.width() as i32;
        let height = self.image.height() as i32;

        Rect {
            x: match self.origin.x {
                Origin::Min => Line::new(x, x + width),
                Origin::Max => Line::new(x - width + 1, x + 1),
            },

            y: match self.origin.y {
                Origin::Min => Line::new(y, y + height),
                Origin::Max => Line::new(y - height + 1, y + 1),
            },
        }
    }

    pub fn rect(&self, origin: Pos<i32>, canvas_size: Size<u16>) -> Option<SpriteRect> {
        let rect = self.bounding_box();

        let x_clip = Clip::new(canvas_size.width, rect.x.translate(origin.x))?;
        let y_clip = Clip::new(
            canvas_size.width,
            rect.y.translate(origin.y - self.position.y * 2),
        )?;

        Some(SpriteRect {
            image: self.image,
            image_offset: Pos::new(x_clip.image_offset, y_clip.image_offset),
            rect: Rect {
                x: x_clip.position,
                y: y_clip.position,
            },
        })
    }

    pub fn collide(&self, other: &Sprite) -> bool {
        let box_a = self.bounding_box();
        let box_b = other.bounding_box();

        let Some(intersection) = box_a.intersect(box_b) else {
            return false;
        };

        for y in intersection.y.range() {
            let mut row_a = self.image.pixels[(y - box_a.y.start) as usize];
            let mut row_b = other.image.pixels[(y - box_b.y.start) as usize];

            if box_a.x.start < box_b.x.start {
                row_a >>= box_b.x.start - box_a.x.start;
            } else {
                row_b >>= box_a.x.start - box_b.x.start;
            }

            if row_a & row_b != 0 {
                return true;
            }
        }

        false
    }
}

impl Image {
    pub fn width(&self) -> u16 {
        self.width
    }
    pub fn height(&self) -> u16 {
        self.pixels.len() as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sprite_collision() {
        let a = Sprite {
            position: Pos::new(0, 0),
            origin: Pos::new(Origin::Min, Origin::Min),
            image: Image {
                pixels: &[0b1111],
                width: 4,
                color: Color::Red,
            },
        };

        let mut b = Sprite {
            position: Pos::new(3, 0),
            origin: Pos::new(Origin::Min, Origin::Min),
            image: Image {
                pixels: &[0b1111, 0b0001],
                width: 4,
                color: Color::Red,
            },
        };

        assert!(a.collide(&b));
        assert!(b.collide(&a));

        // Change Position

        b.position = Pos::new(4, 0);
        assert!(!a.collide(&b));

        b.position = Pos::new(0, -1);
        assert!(a.collide(&b));
        assert!(b.collide(&a));

        b.position = Pos::new(-1, -1);
        assert!(!a.collide(&b));
        assert!(!b.collide(&a));

        // Change Origin

        b.position = Pos::new(-1, 0);
        b.origin = Pos::new(Origin::Max, Origin::Max);
        assert!(!a.collide(&b));

        b.position = Pos::new(3, -1);
        b.origin = Pos::new(Origin::Min, Origin::Max);
        assert!(a.collide(&b));

        b.position = Pos::new(4, -1);
        b.origin = Pos::new(Origin::Min, Origin::Max);
        assert!(!a.collide(&b));
    }

    #[test]
    fn bounding_box() {
        let image = Image {
            pixels: &[0b1111, 0b1000],
            width: 4,
            color: Color::Red,
        };

        let mut sprite = Sprite {
            position: Pos::new(3, 1),
            origin: Pos::new(Origin::Min, Origin::Min),
            image,
        };

        assert_eq!(
            sprite.bounding_box(),
            Rect {
                x: Line::new(3, 7),
                y: Line::new(1, 3),
            }
        );

        sprite.origin = Pos::new(Origin::Max, Origin::Max);
        assert_eq!(
            sprite.bounding_box(),
            Rect {
                x: Line::new(0, 4),
                y: Line::new(0, 2),
            }
        );
    }
}
