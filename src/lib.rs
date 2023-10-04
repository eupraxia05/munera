pub mod eng;
pub mod gfx;
pub mod math;
pub mod ed;

use egui::ComboBox;
use egui::{epaint::Hsva, Context, Response};
use eng::*;
use hecs::{Entity, World, EntityRef, EntityBuilder};
use hecs::serialize::row::{self, SerializeContext, DeserializeContext};
use serde::de::Visitor;
use serde_json::de::StrRead;
use std::any::TypeId;
use std::fs::File;
use std::io::{Write, BufReader, Read};
use serde::ser::SerializeMap;
use mac::define_comps;

/*struct EditorContext {
  selected_ent: Option<Entity>,
  world: World,
  selected_comp_type: usize
}

impl EditorContext {
  pub fn new() -> Self {
    EditorContext {
      selected_ent: None,
      world: World::new(),
      selected_comp_type: 0
    }
  }
}

struct SerContext<'a> {
  engine: &'a mut Engine
}

impl<'a> SerializeContext for SerContext<'a> {
  fn serialize_entity<S>(&mut self, entity: EntityRef<'_>, mut map: S) -> Result<S::Ok, S::Error>
    where S: SerializeMap 
  {
    let comp_types = self.engine.get_comp_types();

    entity.component_types().into_iter().for_each(|t| {
      let ty = comp_types.iter().find(|ty| ty.type_id == t).expect("Failed to find type!");

      map.serialize_entry(&ty.name, (ty.ent_ser)(entity, map));
    });

    if let Some(name) = entity.get::<&NameComp>() {
      map.serialize_entry("NameComp", &*name);
    }

    map.end()
  }
}

struct DeVisitor;

struct DeContext;

impl DeserializeContext for DeContext {
  fn deserialize_entity<'de, M>(&mut self, mut map: M, entity: &mut hecs::EntityBuilder) 
    -> Result<(), M::Error>
    where M: serde::de::MapAccess<'de> 
  {
    while let Some(key) = map.next_key()? {
      match key {
        "NameComp" => {
          entity.add::<NameComp>(map.next_value()?);
        }
        _ => { }
      }
    }

    Ok(())
  }
}

fn ui_callback<'a>(context: &Context, ed_context: &mut EditorContext, engine: &mut Engine) {
  egui::Window::new("Editor").show(context, |ui| {
    {
      ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
          let mut file = File::create("out.json").expect("Failed to open file!");
          let mut ser = serde_json::Serializer::new(&file);
          let mut ctx = SerContext { };
          row::serialize(&ed_context.world, &mut ctx, &mut ser).expect("Failed to serialize!");
          file.flush();
        }
         if ui.button("Load").clicked() {
          let file = File::open("out.json").expect("Failed to open file!");
          let mut contents = String::from("");
          let mut reader = BufReader::new(file);
          reader.read_to_string(&mut contents);
          let str_reader = StrRead::new(&contents);
          let mut de = serde_json::Deserializer::new(str_reader);
          let mut ctx = DeContext;
          ed_context.world = row::deserialize(&mut ctx, &mut de).expect("Failed to deserialize world!");
        }
      });
    }
    
    ui.horizontal(|ui| {
      if ui.button("+").clicked() {
        ed_context.world.spawn(());
      }
      if ui.button("-").clicked() && ed_context.selected_ent.is_some() {
        ed_context.world.despawn(ed_context.selected_ent.unwrap());
        ed_context.selected_ent = None;
      }
    });
    
    {
      let mut ents = ed_context.world.iter().map(|ent| ent.entity()).collect::<Vec<Entity>>();
      ents.sort_by(|a, b| a.id().cmp(&b.id()));
      ents.iter().for_each(|ent| {
        let ent_ref = ed_context.world.entity(*ent)
          .expect("Failed to get entity!");
        let ent_str = if ent_ref.has::<NameComp>() {
          let comp = ent_ref.get::<&NameComp>()
            .expect("Failed to get name component!");
          format!("{}: {}", ent.id(), comp.name)
        } else {
          format!("{}: Unnamed", ent.id())
        };
        
        let is_selected = ed_context.selected_ent.is_some() && *ent == ed_context.selected_ent.unwrap();
        
        if ui.selectable_label(is_selected, ent_str).clicked() {
          ed_context.selected_ent = Some(*ent);
        }
      });
    }
    
    ui.separator();
    
    {
      if let Some(selected_ent) = ed_context.selected_ent {
        ui.horizontal(|ui| {
          ui.label("Name");
          
          let sel_ent = ed_context.world.entity(selected_ent)
            .expect("Failed to get selected ent!");
          
          if sel_ent.has::<NameComp>() {
            {
              let mut name = ed_context
              .world
              .get::<&mut NameComp>(selected_ent)
              .expect("Failed to get selected ent name!");
              ui.text_edit_singleline(&mut name.name);
            }
            
            if ui.button("-").clicked() {
              ed_context
              .world
              .remove_one::<NameComp>(sel_ent.entity())
              .expect("Failed to remove name component!");
            }
          } else {
            if ui.button("+").clicked() {
              ed_context
              .world
              .insert_one(
                sel_ent.entity(),
                NameComp {
                  name: "".to_string(),
                },
              )
              .expect("Failed to add name component!");
            }
          }
        });

        let comp_types = engine.get_comp_types();

        let mut comps_to_rem = Vec::new();
        {
          let sel_ent = ed_context.world.entity(selected_ent)
            .expect("Failed to get selected ent!");

          sel_ent.component_types().into_iter().filter(|t| *t != TypeId::of::<NameComp>()).for_each(|t| {
            let ty = comp_types.iter().find(|ty| ty.type_id == t).expect("Failed to find type!");
            ui.collapsing(ty.name.clone(), |ui| {
              if ui.button("-").clicked() {
                comps_to_rem.push(ty);
              }
              ui.label("Skibadee, skibadanger!");
            });
          });
        }

        for comp in comps_to_rem {
          (comp.ent_rem)(&mut ed_context.world, selected_ent);
        }
      }
    }

    ui.separator();

    if let Some(sel_ent) = ed_context.selected_ent {
      ui.horizontal(|ui| {
        let comp_types = engine.get_comp_types();
  
        ComboBox::from_id_source(comp_types[0].clone())
          .selected_text(comp_types[ed_context.selected_comp_type].name.clone())
          .show_ui(ui, |ui| {
            for (i, ty) in comp_types.iter().enumerate() {
              let ent = ed_context.world.entity(sel_ent)
                .expect("Failed to get selected ent!");
              
              if !(ty.ent_has)(ent) {
                ui.selectable_value(&mut ed_context.selected_comp_type, 
                  i, ty.name.clone());
              }
            }
          });
  
        if ui.button("+").clicked() {
          if ed_context.selected_comp_type < comp_types.len() {
            let ty = &comp_types[ed_context.selected_comp_type];
            (ty.ent_add)(&mut ed_context.world, sel_ent);
          }
        }
      });
    }
  });
}

pub fn run_editor() {
  let mut engine = Engine::new();
  let mut ed_context = EditorContext::new();
  
  let mut gfx = gfx::GfxRuntime::new();
  gfx.get_egui()
  .set_ui_callback(move |ctx| ui_callback(ctx, &mut ed_context, &mut engine));
  gfx.window_loop();
}*/

#[cfg(test)]
mod tests {
  /*use crate::ecs::*;
  
  #[test]
  fn ent_basics() {
    let mut reg = Registry::new();
    let ent = reg.new_ent();
    assert!(reg.has_ent(&ent));
    reg.add_comp(&ent, NameComp::new("Skibadee"));
    assert!(reg.has_comp::<NameComp>(&ent));
    if let Some(comp) = reg.get_comp::<NameComp>(&ent) {
      assert!(comp.0 == "Skibadee")
    } else {
      assert!(false);
    }
    reg.del_comp::<NameComp>(&ent);
    assert!(!reg.has_comp::<NameComp>(&ent));
    assert!(reg.get_comp::<NameComp>(&ent).is_none());
    reg.add_comp(&ent, NameComp::new("Skibadanger"));
    assert!(reg.has_comp::<NameComp>(&ent));
    assert!(reg.get_comp::<NameComp>(&ent).is_some());
    reg.del_ent(&ent);
    assert!(!reg.has_ent(&ent));
    assert!(!reg.has_comp::<NameComp>(&ent));
    assert!(reg.get_comp::<NameComp>(&ent).is_none());
  }
  
  /*#[test]
  fn ent_destroy() {
    let mut scene = crate::ent::Scene::new();
    assert!(scene.get_num_entities() == 0);
    let ent_ref = scene.new_ent();
    let ent = scene.borrow_ent_by_ref(&ent_ref);
    assert!(scene.get_num_entities() == 1);
    scene.destroy_ent(&ent_ref);
    assert!(scene.get_num_entities() == 0);
    assert!(ent.is_none());
  }*/
  
  /*#[test]
  fn ent_find() {
    let mut scene = crate::ent::Scene::new();
    let weak_ent = scene.new_ent();
    let ent = weak_ent.upgrade().expect("Invalid entity!");
    ent.borrow().name.replace(String::from("Skibadee"));
    assert!(ent.borrow().name.borrow().eq(&String::from("Skibadee")));
    let found_ent = scene.find_ent_by_name(&String::from("Skibadee"));
    assert!(found_ent.is_some());
    assert!(Weak::ptr_eq(&found_ent.unwrap(), &weak_ent));
  }*/
  
  /*#[test]
  fn ent_parent() {
    let mut scene = crate::ent::Scene::new();
    let weak_ent1 = scene.new_ent();
    let weak_ent2 = scene.new_ent();
    let ent1 = weak_ent1.upgrade().unwrap();
    let ent2 = weak_ent2.upgrade().unwrap();
    /*ent1.set_parent(&weak_ent2.);
    assert!(ent1.parent.borrow().is_some());
    assert!(ent1.parent.borrow().clone().unwrap().ptr_eq(&weak_ent2));*/
    assert!(ent1.borrow().get_id() == 0);
    assert!(ent2.borrow().get_id() == 1);
    //let children = ent2.get_children();
  }*/ */
}
