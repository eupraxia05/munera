pub trait CompInspect {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool;
}

pub fn inspect_string(label: &mut String, ui: &mut egui::Ui) -> bool {
  let mut modified = false;
  ui.horizontal(|ui| {
    ui.label("Name");
    if ui.text_edit_singleline(label).changed() {
      modified = true;
    }
  });
  modified
}