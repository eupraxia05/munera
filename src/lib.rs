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

mod misc_comps;
pub use misc_comps::*;

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

  #[derive(mac::Comp, serde::Serialize, serde::Deserialize, Clone, Copy, Default)]
  struct FooComp {

  }

  impl crate::editor::inspect::CompInspect for FooComp {
    fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
      false
    }
  }

  #[test]
  fn comp_type_registration() {
    let mut found_comp_type = false;
    for comp_type in inventory::iter::<crate::engine::CompType>() {
      if comp_type.type_id == std::any::TypeId::of::<FooComp>() {
        found_comp_type = true;
        break;
      }
    }
    assert!(found_comp_type);
  }
}
