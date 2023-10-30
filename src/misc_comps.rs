/// Utility component to tag entities with a human-friendly name.
#[derive(mac::Comp, serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct NameComp {
  pub name: String
}

impl crate::editor::inspect::CompInspect for NameComp {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    crate::editor::inspect::inspect_string(&mut self.name, ui)
  }
}

#[derive(mac::Comp, Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct TransformComp {
  pub position: crate::math::Vec3f
}

impl crate::editor::inspect::CompInspect for TransformComp {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    let mut modified = false;
    ui.horizontal(|ui| {
      ui.label("Position");
      let drag_value_x = egui::DragValue::new(&mut self.position.x);
      let drag_value_y = egui::DragValue::new(&mut self.position.y);
      let drag_value_z = egui::DragValue::new(&mut self.position.z);
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