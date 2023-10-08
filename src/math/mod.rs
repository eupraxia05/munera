pub struct Vec2i {
  pub x : i32,
  pub y : i32
}

impl Vec2i {
  pub fn new(x : i32, y : i32) -> Self {
    Self { x: x, y: y }
  }
}

pub struct Vec2u {
  pub x : u32,
  pub y : u32
}

impl Vec2u {
  pub fn new(x : u32, y : u32) -> Self {
    Self { x: x, y: y }
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