use std::ops::{DivAssign, MulAssign};

/// A two-dimensional integer vector.
#[derive(PartialEq, Eq, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
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

impl<T> From<winit::dpi::PhysicalPosition<T>> for Vec2i
  where T : num_traits::cast::AsPrimitive<i32>
{
  fn from(value: winit::dpi::PhysicalPosition<T>) -> Self {
    Self { x: value.x.as_(), y: value.y.as_() }
  }
}

impl<T> From<winit::dpi::PhysicalSize<T>> for Vec2i
  where T : num_traits::cast::AsPrimitive<i32>
{
  fn from(value: winit::dpi::PhysicalSize<T>) -> Self {
    Self { x: value.width.as_(), y: value.height.as_() }
  }
}

impl Into<winit::dpi::Position> for Vec2i {
  fn into(self) -> winit::dpi::Position {
    winit::dpi::Position::Physical(winit::dpi::PhysicalPosition::<i32>::new(self.x, self.y))
  }
}

#[derive(bytemuck::NoUninit, Copy, Clone, serde::Deserialize, serde::Serialize)]
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

impl Default for Vec2u {
  fn default() -> Self {
    Self { x: 0, y: 0 }
  }
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

impl Into<winit::dpi::Size> for Vec2u {
  fn into(self) -> winit::dpi::Size {
    winit::dpi::Size::Physical(winit::dpi::PhysicalSize::<u32>::new(self.x, self.y))
  }
}

impl<T> From<winit::dpi::PhysicalPosition<T>> for Vec2u
  where T : num_traits::cast::AsPrimitive<u32>
{
  fn from(value: winit::dpi::PhysicalPosition<T>) -> Self {
    Self { x: value.x.as_(), y: value.y.as_() }
  }
}

impl<T> From<winit::dpi::PhysicalSize<T>> for Vec2u
  where T : num_traits::cast::AsPrimitive<u32>
{
  fn from(value: winit::dpi::PhysicalSize<T>) -> Self {
    Self { x: value.width.as_(), y: value.height.as_() }
  }
}

impl std::ops::Mul<u32> for Vec2u {
  type Output = Vec2u;

  fn mul(self, rhs: u32) -> Self::Output {
    Vec2u { x: self.x * rhs, y: self.y * rhs }
  }
}

impl std::ops::Div<u32> for Vec2u {
  type Output = Vec2u;

  fn div(self, rhs: u32) -> Self::Output {
    Vec2u { x: self.x / rhs, y: self.y / rhs }
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

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, serde::Serialize, serde::Deserialize)]
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

impl Default for Vec3f {
  fn default() -> Self {
    Self {x: 0.0f32, y: 0.0f32, z: 0.0f32}
  }
}

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, serde::Serialize, serde::Deserialize)]
#[repr(C)]
#[derive(RTTI)]
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

impl Default for Color {
  fn default() -> Self {
    Self { r: 0.0f32, g: 0.0f32, b: 0.0f32, a: 0.0f32 }
  }
}

impl Into<wgpu::Color> for Color {
  fn into(self) -> wgpu::Color {
    wgpu::Color { r: self.r as f64, g: self.g as f64, b: self.b as f64, a: self.a as f64 }
  }
}