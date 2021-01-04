use std::ops::{Add, AddAssign, Sub};

pub type Scalar = f32;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Position {
    pub x: Scalar,
    pub y: Scalar,
}

impl Position {
    pub fn new(x: Scalar, y: Scalar) -> Position {
        Position { x, y }
    }

    pub fn zero() -> Position {
        Position::new(0.0, 0.0)
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Position) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Size {
    pub width: Scalar,
    pub height: Scalar,
}

impl Size {
    pub fn new(width: Scalar, height: Scalar) -> Size {
        Size { width, height }
    }

    pub fn zero() -> Size {
        Size::new(0.0, 0.0)
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl<P: winit::dpi::Pixel> From<winit::dpi::LogicalSize<P>> for Size {
    fn from(size: winit::dpi::LogicalSize<P>) -> Self {
        Size::new(size.width.cast(), size.height.cast())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rect {
    pub origin: Position,
    pub size: Size,
}

impl Rect {
    pub fn new(origin: Position, size: Size) -> Rect {
        Rect { origin, size }
    }

    pub fn contains(&self, position: Position) -> bool {
        position.x >= self.origin.x
            && position.x < self.origin.x + self.size.width
            && position.y >= self.origin.y
            && position.y < self.origin.y + self.size.height
    }

    pub fn center(&self, size: Size) -> Rect {
        let dx = (self.size.width - size.width) / 2.0;
        let dy = (self.size.height - size.height) / 2.0;
        Rect::new(Position::new(self.origin.x + dx, self.origin.y + dy), size)
    }
}
