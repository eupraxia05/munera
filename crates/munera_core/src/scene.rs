#[derive(munera_macros::Asset)]
pub struct SceneAsset {
  name: String,
  world: hecs::World,
}

impl serde_binary::Decode for SceneAsset {
  fn decode(&mut self, de: &mut serde_binary::Deserializer) -> serde_binary::Result<()> {
    Err(serde_binary::Error::Custom("Not implemented yet.".to_string()))
  }
}

impl serde_binary::Encode for SceneAsset {
  fn encode(&self, ser: &mut serde_binary::Serializer) -> serde_binary::Result<()> {
    Err(serde_binary::Error::Custom("Not implemented yet.".to_string())) 
  }
}

pub struct HecsEntSerializeContext<'a> {
  world: &'a hecs::World
}

impl<'a> HecsEntSerializeContext<'a> {
  fn new(world: &'a hecs::World) -> Self {
    Self { world }
  }
}

impl<'a> hecs::serialize::row::SerializeContext for HecsEntSerializeContext<'a> {
  fn serialize_entity<S>(&mut self, entity: hecs::EntityRef<'_>, mut map: S) -> std::result::Result<S::Ok, S::Error>
    where S: serde::ser::SerializeMap 
  {
    let mut comps = Vec::new();
    for comp_type in inventory::iter::<crate::engine::CompType>() {
      if (comp_type.ent_has)(entity) {
        comps.push((comp_type.ent_get)(self.world, entity.entity()));
      }
    }

    map.serialize_entry("comps", &comps)?;

    map.end()
  }
}

impl serde::Serialize for SceneAsset {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: serde::Serializer 
  {
    let mut ctx = HecsEntSerializeContext::new(&self.world);
    hecs::serialize::row::serialize(&self.world, &mut ctx, serializer)
  }
}

pub struct HecsEntDeserializeContext;

impl HecsEntDeserializeContext {
  fn new() -> Self {
    Self { }
  }
}

impl hecs::serialize::row::DeserializeContext for HecsEntDeserializeContext {
  fn deserialize_entity<'de, M>(&mut self, mut map: M, entity: &mut hecs::EntityBuilder,) 
    -> std::result::Result<(), M::Error>
    where M: serde::de::MapAccess<'de> 
  {
    while let Some((key, value)) = map.next_entry::<String, Vec<Box<dyn crate::engine::Comp>>>()? {
      if key == "comps" {
        for comp in value {
          for comp_type in inventory::iter::<crate::engine::CompType>() {
            if (*comp).as_any().type_id() == comp_type.type_id {
              (comp_type.ent_deserialize)(entity, &comp);
            }
          }
        } 
      }
    }

    Ok(())
  }
}

impl<'de> serde::Deserialize<'de> for SceneAsset {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where D: serde::Deserializer<'de> 
  {
    let mut ass = SceneAsset::default();
    ass.world = hecs::serialize::row::deserialize(&mut HecsEntDeserializeContext { }, deserializer)?;
    Ok(ass)
  }
}

impl munera_assets::AssetCreateTabViewer for SceneAsset {
  fn create_tab_viewer(&self) -> Box<dyn munera_assets::AssetTabViewer> {
    Box::new(SceneAssetTabViewer::new())
  }
}

impl munera_assets::AssetPostLoad for SceneAsset {
  fn post_load(&mut self, _name: &str) {
    
  }
}

impl Default for SceneAsset {
  fn default() -> Self {
    Self {
      name: "".to_string(),
      world: hecs::World::new()
    }
  }
}

struct SceneAssetTabViewer {
  selected_ent: Option<hecs::Entity>,
  requested_size: munera_math::Vec2u,
  curr_size: munera_math::Vec2u,
  scene_render_tex: Option<wgpu::Texture>,
  scene_render_tex_id: Option<egui::TextureId>,
  renderer: Option<crate::iso_renderer::IsoRenderer>
}

impl SceneAssetTabViewer {
  fn new() -> Self {
    Self { 
      selected_ent: None,
      requested_size: munera_math::Vec2u::new(0, 0),
      curr_size: munera_math::Vec2u::new(0, 0),
      scene_render_tex: None,
      scene_render_tex_id: None,
      renderer: None,
    }
  }

  fn update_scene_render_tex(&mut self, device: &wgpu::Device, 
    egui_rpass: &mut egui_wgpu_backend::RenderPass) 
  {
    if self.requested_size.x == 0 || self.requested_size.y == 0 {
      return;
    }

    let is_up_to_date = self.scene_render_tex.is_some() && self.requested_size == self.curr_size;

    if !is_up_to_date {
      if self.scene_render_tex.is_some() {
        let mut delta = egui::TexturesDelta::default();
        delta.free.push(self.scene_render_tex_id.unwrap());
        egui_rpass.remove_textures(delta);
        self.scene_render_tex.as_mut().unwrap().destroy();
        self.scene_render_tex = None
      }

      let tex_desc = wgpu::TextureDescriptor {
        label: Some("PlayDockable"),
        size: wgpu::Extent3d { 
          width: self.requested_size.x,
          height: self.requested_size.y,
          depth_or_array_layers: 1
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba16Float]
      };

      let tex = device.create_texture(&tex_desc);
      let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
      self.scene_render_tex_id = Some(egui_rpass.egui_texture_from_wgpu_texture(device, &view, 
        wgpu::FilterMode::Nearest));
      self.scene_render_tex = Some(tex);
      self.curr_size = self.requested_size;
    }
  }

  fn update_iso_renderer(&mut self, device: &wgpu::Device) {
    if self.renderer.is_some() {
      return;
    }

    self.renderer = Some(crate::iso_renderer::IsoRenderer::new(device, wgpu::ColorTargetState{
      format: wgpu::TextureFormat::Rgba16Float,
      blend: None,
      write_mask: wgpu::ColorWrites::ALL
    }))
  }
}

impl munera_assets::AssetTabViewer for SceneAssetTabViewer {
  fn tick(&mut self, asset: &mut dyn munera_assets::Asset, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass,
    output_tex_view: &wgpu::TextureView, queue: &wgpu::Queue) 
  {
    if let Some(scene) = asset.as_any_mut().downcast_mut::<SceneAsset>() {
      self.update_scene_render_tex(device, egui_rpass);
      self.update_iso_renderer(device);

      if let Some(tex) = &self.scene_render_tex {
        if let Some(renderer) = &mut self.renderer {
          let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
            label: Some("SceneAssetTabViewer scene render" )
          });
      
          renderer.render(&scene.world, device, &mut encoder, &tex.create_view(&wgpu::TextureViewDescriptor::default()), self.curr_size);
      
          queue.submit(std::iter::once(encoder.finish()));
        }
      }
    }
  }

  fn build_dockable_content(&mut self, asset: &mut dyn munera_assets::Asset, ui: &mut egui::Ui) -> bool {    
    let mut is_modified = false;
    if let Some(scene) = asset.as_any_mut().downcast_mut::<SceneAsset>() {
      egui::SidePanel::right("ent_comp_list").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
          if ui.button("+ Ent").clicked() {
            scene.world.spawn(());
            is_modified = true;
          } else if self.selected_ent.is_some() && ui.button("- Ent").clicked() {
            scene.world.despawn(self.selected_ent.unwrap());
            is_modified = true;
            self.selected_ent = None;
          }
        });
        
        let mut ents = scene.world.iter().collect::<Vec<hecs::EntityRef>>();
        ents.sort_by(|a, b| {
          if a.entity().id() < b.entity().id() {
            std::cmp::Ordering::Less
          } else if a.entity().id() > b.entity().id() {
            std::cmp::Ordering::Greater
          } else {
            std::cmp::Ordering::Equal
          }
        });

        for ent in ents {
          let mut name = "Unnamed".to_string();
          if ent.has::<crate::NameComp>() {
            name = ent.get::<&crate::NameComp>().unwrap().name.clone();
          }
          let is_selected = self.selected_ent.is_some() && self.selected_ent.unwrap() == ent.entity();
          if ui.selectable_label(is_selected, format!("{}: {}", ent.entity().id(), name)).clicked() {
            self.selected_ent = Some(ent.entity())
          }
        }

        ui.separator();

        if let Some(selected_ent) = self.selected_ent {
          let mut remove_name = false;
          let mut add_name = false;
          if let Ok(ent) = scene.world.entity(selected_ent) {
            if ent.has::<crate::NameComp>() {
              if let Some(mut name_comp) = ent.get::<&mut crate::NameComp>() {
                ui.horizontal(|ui| {
                  if ui.text_edit_singleline(&mut name_comp.name).changed() {
                    is_modified = true;
                  }
                  if ui.button("-").clicked() {
                    remove_name = true;
                  }
                });
              }
            } else {
              if ui.button("+").clicked() {
                add_name = true;
              }
            }
          }

          if add_name {
            scene.world.insert_one(selected_ent, crate::NameComp { name: "".to_string() }).expect("Failed to add name!");
            is_modified = true;
          }

          if remove_name {
            scene.world.remove_one::<crate::NameComp>(selected_ent).expect("Failed to remove name!");
            is_modified = true;
          }

          let mut sel_ent_comp_types: Vec<&crate::engine::CompType> = Vec::new();
          {
            let ent = scene.world.entity(selected_ent).unwrap();
            for comp_typeid in ent.component_types() {
              for comp_type in inventory::iter::<crate::engine::CompType> {
                if comp_type.type_id == comp_typeid && comp_type.name != String::from("NameComp") {
                  sel_ent_comp_types.push(comp_type);
                  break;
                }
              }
            }
          }

          for comp_type in sel_ent_comp_types {
            ui.collapsing(comp_type.name.clone(), |ui| {
              if ui.button("-").clicked() {
                is_modified = true;
                (comp_type.ent_rem)(&mut scene.world, selected_ent);
              }

              if (comp_type.ent_inspect)(&mut scene.world, selected_ent, ui) {
                is_modified = true;
              }
            });
          }

          let mut selected_comp = None;

          egui::ComboBox::new("add_component", "").selected_text("Add Comp").show_ui(ui, |ui| {
            for comp_type in inventory::iter::<crate::engine::CompType> {
              if comp_type.name != String::from("NameComp") {
                if ui.selectable_value(&mut selected_comp, Some(comp_type), comp_type.name.clone()).clicked() {
                  is_modified = true;
                  (comp_type.ent_add)(&mut scene.world, selected_ent);
                }
              }
            }
          });
        }
      });

      egui::CentralPanel::default().show_inside(ui, |ui| {
        self.requested_size = ui.available_size().into();
        if let Some(tex_id) = self.scene_render_tex_id {
          ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(tex_id, self.curr_size)));
        }
      });
    }
    is_modified
  }
}