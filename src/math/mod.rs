pub struct Vec2i {
  pub x : i32,
  pub y : i32
}

impl Vec2i {
  pub fn new(x : i32, y : i32) -> Self {
    Self { x: x, y: y }
  }
}