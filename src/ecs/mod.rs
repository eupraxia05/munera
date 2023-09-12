use std::cell::{RefMut, RefCell};
use std::any::Any;
use std::collections::HashSet;
use std::string::String;
use std::rc::Rc;
use std::ops::Deref;

use gl::GetNamedBufferPointerv;

pub struct EntId(pub usize);

struct CompStorage<CompType>(RefCell<Vec<Option<Rc<RefCell<CompType>>>>>);

trait CompStorageBase {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn push_none(&mut self);
  fn erase(&mut self, index: usize);
}

impl<CompType: 'static> CompStorageBase for CompStorage<CompType> {
  fn as_any(&self) -> &dyn Any {
    self as &dyn Any
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self as &mut dyn Any
  }

  fn push_none(&mut self) {
    self.0.get_mut().push(None)
  }

  fn erase(&mut self, index: usize) {
    let mut vec = self.0.borrow_mut();
    if index < vec.len() {
      vec[index] = None;
    }
  }
}

impl<CompType: 'static> CompStorage<CompType> {
  fn new(capacity: usize) -> Self {
    return Self(RefCell::new(Vec::with_capacity(capacity)));
  }
}

pub struct CompRef<CompType>(Rc<RefCell<CompType>>);

impl<CompType> Deref for CompRef<CompType> {
  type Target = RefCell<CompType>;

  fn deref(&self) -> &Self::Target {
    self.0.deref()
  }
}

pub struct Registry {
  num_ents: usize,
  comp_vecs: Vec<Box<dyn CompStorageBase>>,
  deleted_ents: HashSet<usize>
}

impl Registry {
  pub fn new() -> Self {
    Self {
      num_ents: 0,
      comp_vecs: Vec::new(),
      deleted_ents: HashSet::new()
    }
  }

  pub fn new_ent(&mut self) -> EntId {
    let ent_id = self.num_ents;
    for comp_vec in self.comp_vecs.iter_mut() {
      comp_vec.push_none();
    }
    self.num_ents += 1;
    EntId(ent_id)
  }

  pub fn has_ent(&self, ent: &EntId) -> bool {
    self.num_ents > ent.0 && !self.deleted_ents.contains(&ent.0)
  }

  pub fn del_ent(&mut self, ent: &EntId) {
    if self.has_ent(ent) {
      for comp_vec in self.comp_vecs.iter_mut() {
        comp_vec.erase(ent.0);
      }

      self.deleted_ents.insert(ent.0);
    }
  }

  pub fn add_comp<CompType: 'static>(&mut self, ent: &EntId, comp: CompType) {
    for comp_vec in self.comp_vecs.iter_mut() {
      if let Some(comp_vec) = comp_vec.as_any_mut().
        downcast_mut::<CompStorage<CompType>>() 
      {
        comp_vec.0.borrow_mut()[ent.0] = Some(Rc::new(RefCell::new(comp)));
        return;
      }
    }

    let mut new_comp_storage = CompStorage::<CompType>::new(self.num_ents);

    for _ in 0..self.num_ents {
      new_comp_storage.push_none();
    }

    new_comp_storage.0.borrow_mut()[ent.0] = Some(Rc::new(RefCell::new(comp)));
    self.comp_vecs.push(Box::new(new_comp_storage));
  }

  pub fn get_comp<CompType: 'static>(&self, ent: &EntId) 
    -> Option<CompRef<CompType>>
  {
    if !self.has_ent(ent)
    {
      return None;
    }

    for comp_vec in self.comp_vecs.iter() {
      if let Some(comp_vec) = comp_vec.as_any()
        .downcast_ref::<CompStorage<CompType>>()
      {
        let comp_vec = comp_vec.0.borrow();
        assert!(ent.0 < comp_vec.len());

        if comp_vec[ent.0].is_none() {
          return None;
        }

        return Some(CompRef::<CompType>(comp_vec[ent.0].clone().unwrap()));
      }
    }

    None
  }

  pub fn del_comp<CompType: 'static>(&mut self, ent: &EntId) {
    if !self.has_ent(ent) {
      return;
    }

    for comp_vec in self.comp_vecs.iter() {
      if let Some(comp_vec) = comp_vec.as_any()
        .downcast_ref::<CompStorage<CompType>>()
      {
        let mut comp_vec = comp_vec.0.borrow_mut();
        assert!(ent.0 < comp_vec.len());
        comp_vec[ent.0] = None;
      }
    }
  }

  pub fn has_comp<CompType: 'static>(&self, ent: &EntId) -> bool {
    if !self.has_ent(ent) {
      return false;
    }

    for comp_vec in self.comp_vecs.iter() {
      if let Some(comp_vec) = comp_vec.as_any()
        .downcast_ref::<CompStorage<CompType>>()
      {
        let comp_vec = comp_vec.0.borrow();
        assert!(ent.0 < comp_vec.len());
        return comp_vec[ent.0].is_some();
      }
    }

    false
  }

  pub fn iter(&self) -> RegistryIterator {
    RegistryIterator::new(self)
  }
}

pub struct RegistryIterator<'a> {
  ent: usize,
  reg: &'a Registry
}

impl<'a> RegistryIterator<'a> {
  fn new(reg: &'a Registry) -> Self {
    Self { ent: 0, reg: reg }
  }
}

impl<'a> Iterator for RegistryIterator<'a> {
  type Item = EntId;

  fn next (&mut self) -> Option<Self::Item> {
    while self.ent < self.reg.num_ents {
      self.ent += 1;
      if self.reg.has_ent(&EntId(self.ent)){
        return Some(EntId(self.ent));
      }
    }
    
    None
  }
}

pub struct NameComp(pub String);

impl NameComp {
  pub fn new(str: &str) -> Self {
    NameComp(String::from(str))
  }
}