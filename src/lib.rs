pub mod ecs;
pub mod gfx;
pub mod math;

use egui::{Context, epaint::Hsva, Response};
use ecs::*;
use std::rc::Rc;
use std::cell::RefCell;

struct EditorContext {
    selected_ent: Option<EntId>,
    test_str: String
}

impl EditorContext {
    pub fn new() -> Self {
        EditorContext { selected_ent: None, test_str: String::from("") }
    }
}

fn ui_callback(context: &Context, registry: Rc<RefCell<Registry>>, ed_context: Rc<RefCell<EditorContext>>) {
    egui::Window::new("Editor").show(context, |ui| 
    {
        let mut reg = registry.borrow_mut();
        let mut ed = ed_context.borrow_mut();

        ui.text_edit_singleline(&mut ed.test_str);

        if ui.button("Add Ent").clicked() {
            reg.new_ent();
        }

        reg.iter().for_each(|ent| {
            let ent_str = if let Some(name) = reg.get_comp::<NameComp>(&ent) {
                format!("{}: {}", ent.0, name.borrow().0)
            } else {
                format!("{}: Unnamed", ent.0)
            };

            let is_selected = ed.selected_ent.is_some() && ed.selected_ent.as_mut().unwrap().0 == ent.0;

            if ui.selectable_label(is_selected, ent_str).clicked() {
                ed.selected_ent = Some(ent);
            }
        });

        ui.separator();

        if let Some(selected_ent) = ed.selected_ent.as_ref() {
            if let Some(name) = reg.get_comp::<NameComp>(&selected_ent) {
                ui.text_edit_singleline(&mut name.borrow_mut().0);
            } else {
                if ui.button("Add Name").clicked() {
                    reg.add_comp::<NameComp>(selected_ent, NameComp::new(""));
                }
            }
        }
    });
}

pub fn run_editor() {
    let registry = Rc::new(RefCell::new(Registry::new()));
    let ed_context = Rc::new(RefCell::new(EditorContext::new()));

    let mut gfx = gfx::GfxRuntime::new();
    gfx.get_egui().set_ui_callback(move |ctx| ui_callback(ctx, registry.clone(), ed_context.clone()));
    gfx.window_loop();
}

fn main() {
    run_editor();
}

#[cfg(test)]
mod tests {
    use crate::ecs::*;

    #[test]
    fn ent_basics() {
        let mut reg = Registry::new();
        let ent = reg.new_ent();
        assert!(reg.has_ent(&ent));
        reg.add_comp(&ent, NameComp::new("Skibadee"));
        assert!(reg.has_comp::<NameComp>(&ent));
        if let Some(comp) = reg.get_comp::<NameComp>(&ent) {
            assert!(comp.borrow().0 == "Skibadee")
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
    }*/
}
