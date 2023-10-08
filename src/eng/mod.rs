use std::any::TypeId;
use hecs::{EntityRef, Entity, World};
use serde::{Serialize, Deserialize, ser::SerializeMap};
use mac::{Comp, define_comps};
use std::cell::RefCell;

use crate::{gfx::GfxRuntime, ass::AssetCache};

pub trait Comp {
  fn ent_has(ent: EntityRef) -> bool;
  fn ent_add(world: &mut World, ent: Entity);
  fn ent_rem(world: &mut World, ent: Entity);
}

#[derive(Comp, Serialize, Deserialize, Default)]
pub struct NameComp {
  pub name: String
}

#[derive(Comp, Serialize, Deserialize, Default)]
pub struct ParentComp {
  pub parent: i32
}

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

pub struct Engine {
  comp_types: Vec<CompType>,
  gfx: RefCell<GfxRuntime>,
  asset_cache: RefCell<AssetCache>
}

impl Engine {
  pub fn new() -> Self {
    return Self { 
      comp_types: define_comps!(), 
      gfx: RefCell::new(GfxRuntime::new()), 
      asset_cache: RefCell::new(AssetCache::new())
    }
  }

  pub fn get_comp_types(&self) -> &Vec<CompType> {
    &self.comp_types
  }

  pub fn get_gfx(&self) -> &RefCell<GfxRuntime> {
    &self.gfx
  }

  pub fn get_asset_cache(&self) -> &RefCell<AssetCache> {
    &self.asset_cache
  }
}