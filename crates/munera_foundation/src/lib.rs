/// A standard Result type used by various engine systems.
pub type Result<T> = std::result::Result<T, Error>;

/// A standard Error type used by various engine systems.
#[derive(Debug, Clone)]
pub struct Error {
  message: String
}

impl Error {
  pub fn new<T>(msg: &T) -> Self where T: ToString + ?Sized {
    return Self {
      message: msg.to_string()
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

pub trait PropertyInspect {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool;
}

impl PropertyInspect for String {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    let mut modified = false;
    if ui.text_edit_singleline(self).changed() {
      modified = true;
    }
    modified
  }
}

impl PropertyInspect for u32 {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    ui.add(egui::DragValue::new(self)).changed()
  }
}

impl PropertyInspect for munera_math::Color {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    let mut modified = false;
    let mut col = &mut [self.r, self.g, self.b, self.a];
    if ui.color_edit_button_rgba_unmultiplied(&mut col).changed() {
      self.r = col[0];
      self.g = col[1];
      self.b = col[2];
      self.a = col[3];
      modified = true;
    }
    modified
  }
}

impl PropertyInspect for f32 {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    ui.add(egui::DragValue::new(self).speed(0.1f32)).changed()
  }
}

impl PropertyInspect for munera_math::Vec3f {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    let mut modified = false;
    modified |= PropertyInspect::inspect(&mut self.x, ui);
    modified |= PropertyInspect::inspect(&mut self.y, ui);
    modified |= PropertyInspect::inspect(&mut self.z, ui);
    modified
  }
}