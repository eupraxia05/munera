/// Utility component to tag entities with a human-friendly name.
#[derive(munera_macros::Comp, serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct NameComp {
  pub name: String
}

impl crate::editor::inspect::CompInspect for NameComp {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    crate::editor::inspect::inspect_string(&mut self.name, ui)
  }
}

#[derive(munera_macros::Comp, Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct TransformComp {
  pub position: crate::math::Vec3f
}

impl TransformComp {
  pub fn obj_to_pix(&self, obj_pos: crate::math::Vec3f) -> crate::math::Vec2i {
    crate::math::Vec2i {
      x: ((self.position.x + obj_pos.x) * 32.0f32 
        + (self.position.y + obj_pos.y) * 32.0f32) as i32,
      y: ((self.position.x + obj_pos.x) * -16.0f32 
        + (self.position.y + obj_pos.y) * 16.0f32
        + (self.position.z + obj_pos.z) * 16.0f32) as i32
    }
  }
}

impl crate::editor::inspect::CompInspect for TransformComp {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    let mut modified = false;
    ui.horizontal(|ui| {
      ui.label("Position");
      let drag_value_x = 
        egui::DragValue::new(&mut self.position.x).speed(0.1f64);
      let drag_value_y = 
        egui::DragValue::new(&mut self.position.y).speed(0.1f64);
      let drag_value_z = 
        egui::DragValue::new(&mut self.position.z).speed(0.1f64);
      if ui.add(drag_value_x).changed() {
        modified = true;
      }
      if ui.add(drag_value_y).changed() {
        modified = true;
      }
      if ui.add(drag_value_z).changed() {
        modified = true;
      }
    });
    modified
  }
}