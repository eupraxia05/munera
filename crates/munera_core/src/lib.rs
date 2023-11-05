//#![feature(trait_upcasting)]

//! A humble little game engine.

/// Core engine features.
pub mod engine;

/// A logger implementation.
pub mod logger;

// An isometric renderer.
pub mod iso_renderer;

mod misc_comps;
pub use misc_comps::*;

mod scene;
pub use scene::*;

mod image;
pub use image::*;

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

#[cfg(test)]
mod tests {

  #[derive(munera_macros::Comp, serde::Serialize, serde::Deserialize, Clone, Copy, 
    Default, rtti_derive::RTTI)]
  struct FooComp {

  }

  /*impl crate::editor::inspect::CompInspect for FooComp {
    fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
      false
    }
  }*/

  #[test]
  fn comp_type_registration() {
    let comp_type = crate::engine::CompType::find::<FooComp>();
    assert!(comp_type.is_some());
    assert!(comp_type.unwrap().type_id == std::any::TypeId::of::<FooComp>())
  }
}
