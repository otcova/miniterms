use num_traits::Num;
use std::ops::{Add, Range, Sub};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pos<T: Copy> {
    pub x: T,
    pub y: T,
}

#[derive(Copy, Clone, Debug)]
pub struct Size<T: Copy> {
    pub width: T,
    pub height: T,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Line<T: Copy> {
    pub start: T,
    /// Exclusive, must be greater than start
    pub end: T,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect<T: Copy> {
    pub x: Line<T>,
    pub y: Line<T>,
}

impl<T: Copy> Pos<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Copy + Num> Size<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl<T: Copy> Line<T> {
    /// start is inclusive
    /// end is exclusive
    pub fn new(start: T, end: T) -> Line<T> {
        Line { start, end }
    }

    pub fn range(self) -> Range<T> {
        self.start..self.end
    }
}

impl<T: Copy + Ord> Line<T> {
    /// Returns None if the size of the line is not greater than 0 (if start >= end)
    ///
    /// start is inclusive
    /// end is exclusive
    pub fn new_checked(start: T, end: T) -> Option<Line<T>> {
        if start >= end {
            return None;
        }

        Some(Line { start, end })
    }

    pub fn intersect(self, other: Line<T>) -> Option<Line<T>> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        Line::new_checked(start, end)
    }
}

impl<T: Copy + Num> Line<T> {
    pub fn size(self) -> T {
        self.end - self.start
    }
    pub fn translate(self, amount: T) -> Self {
        Self {
            start: self.start + amount,
            end: self.end + amount,
        }
    }
}

impl<T: Copy + Ord> Rect<T> {
    pub fn intersect(self, other: Self) -> Option<Self> {
        Some(Rect {
            x: self.x.intersect(other.x)?,
            y: self.y.intersect(other.y)?,
        })
    }
}

impl From<ratatui::layout::Size> for Size<u16> {
    fn from(value: ratatui::layout::Size) -> Self {
        Size {
            width: value.width,
            height: value.height,
        }
    }
}

impl<T: Copy + Num> Rect<T> {
    #[allow(unused)]
    pub fn size(self) -> Size<T> {
        Size::new(self.x.size(), self.y.size())
    }
}

impl<T: Copy + Num> Add for Pos<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: Copy + Num> Sub for Pos<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Copy + Num> Add for Size<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Size::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl<T: Copy + Num> Sub for Size<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Size::new(self.width - rhs.width, self.height - rhs.height)
    }
}

impl<T: Copy + Num> From<Pos<T>> for Size<T> {
    fn from(pos: Pos<T>) -> Size<T> {
        Size {
            width: pos.x,
            height: pos.y,
        }
    }
}
