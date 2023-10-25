/// A two-dimensional integer vector.
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct Vec2i {
  pub x : i32,
  pub y : i32
}

impl Vec2i {
  /// Creates a new vector from two values.
  pub fn new(x : i32, y : i32) -> Self {
    Self { x: x, y: y }
  }
}

impl std::ops::AddAssign for Vec2i {
  fn add_assign(&mut self, rhs: Self) {
    self.x += rhs.x;
    self.y += rhs.y;
  }
}

impl std::ops::SubAssign for Vec2i {
  fn sub_assign(&mut self, rhs: Self) {
    self.x -= rhs.x;
    self.y -= rhs.y;
  }
}

impl std::ops::MulAssign for Vec2i {
  fn mul_assign(&mut self, rhs: Self) {
    self.x *= rhs.x;
    self.y *= rhs.y;
  }
}

impl std::ops::DivAssign for Vec2i {
  fn div_assign(&mut self, rhs: Self) {
    self.x /= rhs.x;
    self.y /= rhs.y;
  }
}

impl std::fmt::Display for Vec2i {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("[{}, {}]", self.x, self.y))
  }
}

impl Default for Vec2i {
  fn default() -> Self {
    Self { x: 0, y: 0 }
  }
}

#[derive(bytemuck::NoUninit, Copy, Clone)]
#[repr(C)]
pub struct Vec2u {
  pub x : u32,
  pub y : u32
}

impl Vec2u {
  pub fn new(x : u32, y : u32) -> Self {
    Self { x, y }
  }
}

impl PartialEq for Vec2u {
  fn eq(&self, other: &Self) -> bool {
    self.x == other.x && self.y == other.y
  }

  fn ne(&self, other: &Self) -> bool {
    self.x != other.x || self.y != other.y
  }
}

impl Eq for Vec2u {

}

impl Into<egui::Vec2> for Vec2u {
  fn into(self) -> egui::Vec2 {
    egui::Vec2::new(self.x as f32, self.y as f32)
  }
}

impl From<egui::Vec2> for Vec2u {
  fn from(value: egui::Vec2) -> Self {
    Self { x: value.x as u32, y: value.y as u32 }

  }
}

pub struct Vec2f {
  pub x: f32,
  pub y: f32
}

impl Vec2f {
  pub fn new(x : f32, y : f32) -> Self {
    Self{x: x, y: y}
  }
}

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Vec3f {
  pub x: f32,
  pub y: f32,
  pub z: f32
}

impl Vec3f {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Self{x: x, y: y, z: z}
  }
}

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Color {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32
}

impl Color {
  pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
    Self {r, g, b, a}
  }
}