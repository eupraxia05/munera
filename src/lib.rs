//#![feature(trait_upcasting)]

//! A humble little game engine.

/// Core engine features.
pub mod engine;

/// A logger implementation.
pub mod logger;

/// Vector, matrix, and spatial math utilities.
pub mod math;

/// Editor implementation.
pub mod editor;

/// Asset system implementation.
pub mod assets;

// An isometric renderer.
pub mod iso_renderer;

/// A standard Result type used by various engine systems.
pub type Result<T> = std::result::Result<T, Error>;

pub use egui;
pub use log;
pub use wgpu;
pub use egui_wgpu_backend;
pub use egui_extras;
pub use winit;

#[macro_use]
extern crate rtti_derive;
extern crate rtti;
use rtti::RTTI;

/// A standard Error type used by various engine systems.
#[derive(Debug, Clone)]
pub struct Error {
  message: String
}

impl Error {
  fn new<T>(msg: &T) -> Self where T: ToString + ?Sized {
    return Self {
      message: msg.to_string()
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

#[cfg(test)]
mod tests {
  
}
