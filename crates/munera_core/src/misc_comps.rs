/// Utility component to tag entities with a human-friendly name.
#[derive(munera_macros::Comp, serde::Serialize, serde::Deserialize, Default, 
  Clone)]
pub struct NameComp {
  pub name: String
}

#[derive(munera_macros::Comp, Default, serde::Serialize, serde::Deserialize, 
  Clone)]
pub struct TransformComp {
  pub position: munera_math::Vec3f
}

impl TransformComp {
  pub fn obj_to_pix(&self, obj_pos: munera_math::Vec3f) -> munera_math::Vec2i {
    munera_math::Vec2i {
      x: ((self.position.x + obj_pos.x) * 32.0f32 
        + (self.position.y + obj_pos.y) * 32.0f32) as i32,
      y: ((self.position.x + obj_pos.x) * -16.0f32 
        + (self.position.y + obj_pos.y) * 16.0f32
        + (self.position.z + obj_pos.z) * 16.0f32) as i32
    }
  }
}
