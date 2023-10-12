use std::any::TypeId;
use hecs::{EntityRef, Entity, World};
use serde::{Serialize, Deserialize, ser::SerializeMap};
use mac::{Comp, define_comps};
use std::cell::RefCell;

use crate::gfx;
use crate::ass;

pub mod eng_log;

// A base trait to generate metadata for a component type.
pub trait Comp {
  fn ent_has(ent: EntityRef) -> bool;
  fn ent_add(world: &mut World, ent: Entity);
  fn ent_rem(world: &mut World, ent: Entity);
}

/// Utility component to tag entities with a human-friendly name.
#[derive(Comp, Serialize, Deserialize, Default)]
pub struct NameComp {
  pub name: String
}

/// Contains metadata defined for a particular component type.
pub struct CompType {
  pub name: String,
  pub type_id: TypeId,
  pub ent_has: fn(EntityRef) -> bool,
  pub ent_add: fn(&mut World, Entity),
  pub ent_rem: fn(&mut World, Entity),
}

impl CompType {
  fn new<T>(name: &str) -> Self
    where T: Comp + 'static {
    Self { name: name.to_string(), type_id: TypeId::of::<T>(), ent_has: T::ent_has, 
      ent_add: T::ent_add, ent_rem: T::ent_rem }
  }
}

/// A context containing all engine systems, assets, and metadata.
pub struct Engine {
  comp_types: Vec<CompType>,
  gfx: RefCell<Box<dyn gfx::Gfx>>,
  asset_cache: RefCell<ass::AssetCache>
}

impl Engine {
  pub fn new() -> Self {
    log::set_logger(&eng_log::LOGGER)
      .map(|()| log::set_max_level(log::LevelFilter::Info))
      .expect("Couldn't set logger!");
    log::info!("Initializing engine...");
    Self { 
      comp_types: define_comps!(), 
      gfx: RefCell::new(Box::new(gfx::OglGfx::new())), 
      asset_cache: RefCell::new(ass::AssetCache::new())
    }
  }

  pub fn get_comp_types(&self) -> &Vec<CompType> {
    &self.comp_types
  }

  pub fn get_gfx(&self) -> &RefCell<Box<dyn gfx::Gfx>> {
    &self.gfx
  }

  pub fn get_asset_cache(&self) -> &RefCell<ass::AssetCache> {
    &self.asset_cache
  }
}