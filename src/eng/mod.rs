use std::any::TypeId;
use hecs::{EntityRef, Entity, World};
use serde::{Serialize, Deserialize, ser::SerializeMap};
use mac::{Comp, define_comps};

use crate::gfx::GfxRuntime;

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
  gfx: GfxRuntime
}

impl Engine {
  pub fn new() -> Self {
    return Self { comp_types: define_comps!(), gfx: GfxRuntime::new() }
  }

  pub fn get_comp_types(&self) -> &Vec<CompType> {
    &self.comp_types
  }

  pub fn get_gfx(&mut self) -> &mut GfxRuntime {
    &mut self.gfx
  }
}