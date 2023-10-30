use std::any::Any;
use std::mem::size_of;
use std::{collections::HashMap, fs};
use egui_extras::RetainedImage;
use serde::ser::SerializeMap;
use serde_binary::{Decode, Deserializer};
use serde_binary::binary_stream::Endian;
use spirv_reflect::ShaderModule;

use crate::{math, engine};
use crate::Result;
use crate::Error;

pub trait Asset : serde_binary::Encode + serde_binary::Decode + Any {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  fn as_asset(&self) -> &dyn Asset;
  fn size_bytes(&self) -> usize;
  fn set_name(&mut self, name: &String);
  fn post_load(&mut self);
  fn get_asset_type_name(&self) -> &str;
  fn create_tab_viewer(&self) -> Box<dyn AssetTabViewer>;
}

pub trait AssetTabViewer {
  fn tick(&mut self, asset: &mut dyn Asset, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass, 
    output_tex_view: &wgpu::TextureView, queue: &wgpu::Queue);
  fn build_dockable_content(&mut self, asset: &mut dyn Asset, ui: &mut egui::Ui) -> bool;
}

pub struct ImageAsset {
  pub size: math::Vec2u,
  pub data: Vec<u8>,
  retained_image: Option<RetainedImage>,
  name: String
}

impl ImageAsset {
  fn new() -> Self {
    Self { 
      size: math::Vec2u::new(0, 0),
      data: Vec::<u8>::new(),
      retained_image: None,
      name: "".to_string()
    }
  }

  pub fn retained_image(&self) -> &Option<RetainedImage> {
    &self.retained_image
  }
}

impl Asset for ImageAsset {
  fn get_asset_type_name(&self) -> &str {
    "Image"
  }
  
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_asset(&self) -> &dyn Asset {
    self
  }

  fn size_bytes(&self) -> usize {
    self.data.len() * size_of::<u8>() + size_of::<math::Vec2u>()
  }

  fn create_tab_viewer(&self) -> Box<dyn AssetTabViewer> {
    Box::new(ImageAssetTabViewer::new())
  }

  fn set_name(&mut self, name: &String) {
    self.name = name.clone();
  }

  fn post_load(&mut self) {
    let col_img = 
        egui::ColorImage::from_rgba_premultiplied(
          [self.size.x as usize, self.size.y as usize], 
          &self.data);
        
    self.retained_image 
      = Some(RetainedImage::from_color_image(self.name.clone(), col_img));
  }
}

impl serde_binary::Encode for ImageAsset {
  fn encode(&self, _ser: &mut serde_binary::Serializer) 
    -> serde_binary::Result<()> 
  {
    Err(serde_binary::Error::Custom("Unimplemented :(".to_string()))
  }
}

impl serde_binary::Decode for ImageAsset {
  fn decode(&mut self, de: &mut Deserializer) -> serde_binary::Result<()> {
    self.size.x = de.reader.read_u32().expect("Failed to read value!");
    self.size.y = de.reader.read_u32().expect("Failed to read value!");
    self.data = de.reader.read_bytes(self.size.y as usize * self.size.x as usize * size_of::<u8>() as usize * 4).expect("Failed to read data!");
    
    Ok(())
  }
}

struct ImageAssetTabViewer {
  zoom: f32
}

impl ImageAssetTabViewer {
  fn new() -> Self {
    Self { zoom: 1.0 }
  }
}

impl AssetTabViewer for ImageAssetTabViewer {
  fn tick(&mut self, asset: &mut dyn Asset, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass, 
    output_tex_view: &wgpu::TextureView, queue: &wgpu::Queue) 
  {
      
  }

  fn build_dockable_content(&mut self, asset: &mut dyn Asset, ui: &mut egui::Ui) -> bool {
    if let Some(img) = asset.as_any().downcast_ref::<ImageAsset>() {
      egui::SidePanel::new(egui::panel::Side::Right, egui::Id::new("AssetEditorDockable")).show_inside(ui, |ui| {
        ui.label(format!("Resolution: {} x {}", img.size.x, img.size.y));
        ui.add(egui::Slider::new(&mut self.zoom, 0.1..=4.0).text("Zoom"));
      });
      egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.centered_and_justified(|ui| {
          let w = (img.size.x as f32 * self.zoom) as u32;
          let h = (img.size.y as f32 * self.zoom) as u32;
          let aspect = w as f32 / h as f32;
          let disp_h = std::cmp::min(ui.available_height() as u32, h);
          let disp_w = std::cmp::min(ui.available_width() as u32, 
            (disp_h as f32 * aspect) as u32);
          let disp_h = std::cmp::min(ui.available_height() as u32, 
            (disp_w as f32 / aspect) as u32);
          ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(img.retained_image.as_ref().unwrap()
            .texture_id(ui.ctx()), egui::Vec2::new(disp_w as f32, disp_h as f32))));
        });
      });
    } else {
      log::error!("ImageAssetTabViewer was used for an asset that wasn't an image!");
    }

    false
  }
}

#[derive(serde_repr::Serialize_repr, Clone, Copy, Debug)]
#[repr(u8)]
pub enum ShaderType {
  Vertex = 1,
  Fragment = 2
}

impl std::fmt::Display for ShaderType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self)
  }
}

pub struct ShaderAsset {
  pub shader_type: ShaderType,
  pub code: Vec<u8>,
  shader_module: Option<ShaderModule>
}

impl ShaderAsset {
  fn new(shader_type: ShaderType, code: &Vec<u8>) -> Self {
    Self { shader_type, code: code.clone(), shader_module: None }
  }
}

impl Asset for ShaderAsset {
  fn get_asset_type_name(&self) -> &str {
    "Shader"
  }

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_asset(&self) -> &dyn Asset {
    self
  }

  fn size_bytes(&self) -> usize {
    size_of::<Self>() + self.code.len() * size_of::<u8>()
  }

  fn create_tab_viewer(&self) -> Box<dyn AssetTabViewer> {
    Box::new(ShaderAssetTabViewer{ })
  }

  fn set_name(&mut self, name: &String) {
    
  }

  fn post_load(&mut self) {
    self.shader_module = spirv_reflect::create_shader_module(&self.code).ok();
  }
}

impl serde_binary::Encode for ShaderAsset {
  fn encode(&self, ser: &mut serde_binary::Serializer) 
    -> serde_binary::Result<()> 
  {
    ser.writer.write_string("Shader")?;
    ser.writer.write_u8(self.shader_type as u8)?;
    ser.writer.write_usize(self.code.len())?;
    ser.writer.write_bytes(&self.code)?;
    Ok(())
  }
}

impl serde_binary::Decode for ShaderAsset {
  fn decode(&mut self, de: &mut Deserializer) -> serde_binary::Result<()> {
    let shader_type = de.reader.read_u8()?;
    self.shader_type = match shader_type {
      1 => ShaderType::Vertex,
      2 => ShaderType::Fragment,
      _ => return Err(serde_binary::Error::Custom(
        "Unrecognized shader type".to_string()))
    };
    let code_len = de.reader.read_usize()?;
    self.code = de.reader.read_bytes(code_len)?;
    Ok(())
  }
}

impl Default for ShaderAsset {
  fn default() -> Self {
      Self { shader_type: ShaderType::Vertex, code: Vec::new(), shader_module: None }
  }
}

struct ShaderAssetTabViewer;

impl AssetTabViewer for ShaderAssetTabViewer {
  fn tick(&mut self, asset: &mut dyn Asset, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass,
    output_tex_view: &wgpu::TextureView, queue: &wgpu::Queue) 
  {
    
  }

  fn build_dockable_content(&mut self, asset: &mut dyn Asset, ui: &mut egui::Ui) -> bool {
    if let Some(shader) = asset.as_any().downcast_ref::<ShaderAsset>() {
      ui.label(format!("Type: {}", shader.shader_type.to_string()));

      let module = shader.shader_module.as_ref().expect("Null shader module!");
      ui.label(format!("Source file: {}", module.get_source_file()));
      ui.label(format!("Source language: {:?}", module.get_source_language()));
      ui.label(format!("Source language version: {}", module.get_source_language_version()));
      ui.collapsing("Input Variables", |ui| {
        if let Ok(input_vars) = module.enumerate_input_variables(None) {
          input_vars.iter().for_each(|v| {
            ui.label(format!("{}: location {}", v.name, v.location));
          });
        }
      });
      ui.collapsing("Output Variables", |ui| {
        if let Ok(output_vars) = module.enumerate_output_variables(None) {
          output_vars.iter().for_each(|v| {
            ui.label(format!("{}: location {}", v.name, v.location));
          });
        }
      });
      ui.collapsing("Descriptor Sets", |ui| {
        if let Ok(descriptor_sets) = module.enumerate_descriptor_sets(None) {
          descriptor_sets.iter().for_each(|d| {
            ui.label(format!("{}", d.set));
          });
        }
      });
    } else {
      log::error!("ShaderAssetTabViewer was used for an asset that wasn't a shader!");
    }

    false
  }
}

pub struct SceneAsset {
  name: String,
  world: hecs::World,
}

impl serde_binary::Decode for SceneAsset {
  fn decode(&mut self, de: &mut Deserializer) -> serde_binary::Result<()> {
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

impl Asset for SceneAsset {
  fn get_asset_type_name(&self) -> &str {
    "Scene"
  }

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }

  fn as_asset(&self) -> &dyn Asset {
    self
  }

  fn create_tab_viewer(&self) -> Box<dyn AssetTabViewer> {
    Box::new(SceneAssetTabViewer::new())
  }

  fn post_load(&mut self) {
    
  }

  fn set_name(&mut self, name: &String) {
    self.name = name.clone();
  }
  
  fn size_bytes(&self) -> usize {
    // not accurate, but not sure how to get the byte size of a world
    return size_of::<hecs::World>();
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
  requested_size: crate::math::Vec2u,
  curr_size: crate::math::Vec2u,
  scene_render_tex: Option<wgpu::Texture>,
  scene_render_tex_id: Option<egui::TextureId>,
  renderer: Option<crate::iso_renderer::IsoRenderer>
}

impl SceneAssetTabViewer {
  fn new() -> Self {
    Self { 
      selected_ent: None,
      requested_size: crate::math::Vec2u::new(0, 0),
      curr_size: crate::math::Vec2u::new(0, 0),
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

impl AssetTabViewer for SceneAssetTabViewer {
  fn tick(&mut self, asset: &mut dyn Asset, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass,
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

  fn build_dockable_content(&mut self, asset: &mut dyn Asset, ui: &mut egui::Ui) -> bool {    
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

pub struct AssetCache {
  assets: HashMap<String, Box<dyn Asset>>
}

impl AssetCache {
  pub fn new() -> Self {
    Self {
      assets: HashMap::new()
    }
  }

  pub fn load_file(&mut self, name: &String) -> Result<()> {
    if !self.assets.contains_key(name) {
      log::info!("Loading {}", name);
      if let Ok(read) = fs::read(name) {
        if read[0] == b'b' {
          match serde_binary::decode::<AssetDeserializeHelper>(&read[1..], Endian::Little) {
            Ok(decode) => {
              let mut ass = decode.asset.unwrap();
              ass.set_name(name);
              ass.post_load();
              self.assets.insert(name.clone(), ass);
              return Ok(());
            },
            Err(err) => {
              return Err(Error::new(&format!("Failed to decode {}: {}", name, err)));
            }
          }
        } else if read[0] == b't' {
          let str = read[1..].to_vec();
          match serde_json::from_slice::<AssetDeserializeHelper>(&str) {
            Ok(deserialize) => {
              let mut ass = deserialize.asset.unwrap();
              ass.set_name(name);
              ass.post_load();
              self.assets.insert(name.clone(), ass);
              return Ok(());
            },
            Err(err) => {
              return Err(Error::new(&format!("Failed to deserialize {}: {}", name, err)));
            }
          }
        } else {
          return Err(Error::new(&format!("Failed to read binary/text descriminator from {}", name)));
        }
        
      } else {
        return Err(Error::new(&format!("Failed to open {}", name)));
      }
    }
    Ok(())
  }

  pub fn borrow_asset<AssetType>(&self, name: &String) -> Option<&AssetType>
    where AssetType: Asset 
  {
    if let Some(ass) = self.assets.get(name) {
      Some((ass.as_any()).downcast_ref::<AssetType>().unwrap())
    } else {
      None
    }
  }

  pub fn borrow_asset_mut<AssetType>(&mut self, name: &String) -> Option<&mut AssetType>
    where AssetType: Asset
  {
    if let Some(ass) = self.assets.get_mut(name) {
      Some((ass.as_any_mut()).downcast_mut::<AssetType>().unwrap())
    } else {
      None
    }
  }

  pub fn borrow_asset_generic_mut(&mut self, name: &String) -> Option<&mut dyn Asset> {
    if let Some(ass) = self.assets.get_mut(name) {
      Some(ass.as_mut())
    } else {
      None
    }
  }

  pub fn borrow_all_assets(&self) -> &HashMap<String, Box<dyn Asset>> {
    &self.assets
  }
}

struct AssetDeserializeHelper {
  asset: Option<Box<dyn Asset>>
}

impl Decode for AssetDeserializeHelper {
  fn decode(&mut self, de: &mut Deserializer) -> serde_binary::Result<()> {
    if let Ok(ass_type) = de.reader.read_string() {
      let mut asset: Box<dyn Asset> = match ass_type.as_str() {
        "Image" => Box::new(ImageAsset::new()),
        "Shader" => Box::new(ShaderAsset::default()),
        _ => {
          return Err(
            serde_binary::Error::Custom(format!("Unrecognized asset type: {}", 
            ass_type)));
        }
      };
      asset.decode(de)?;
      self.asset = Some(asset);
      Ok(())
    } else {
      Err(serde_binary::Error::Custom("Failed to decode asset type".to_string()))
    }
  }
}

struct AssetDeserializeHelperVisitor;

impl<'de> serde::de::Visitor<'de> for AssetDeserializeHelperVisitor {
  type Value = AssetDeserializeHelper;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("a valid asset");
    Ok(())
  }

  fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where A: serde::de::MapAccess<'de>
  {
    let mut asset_type = None;
    let mut asset_value = None;

    while let Some((key, value)) = map.next_entry::<&str, serde_json::Value>()? {
      match key {
        "type" => {
          asset_type = Some(String::from(value.as_str().unwrap()));
        },
        "asset" => {
          asset_value = Some(value);
        },
        &_ => { }
      }
    }

    if asset_type.is_none() {
      return Err(serde::de::Error::missing_field("type"));
    }

    let ass: Box<dyn Asset> = match asset_type.as_ref().unwrap().as_str() {
      "Scene" => {
        Box::new(match serde_json::from_value::<SceneAsset>(asset_value.unwrap()) {
          Ok(ass) => ass,
          Err(err) => return Err(serde::de::Error::custom(err.to_string()))
        })
      }
      _ => {
        return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(asset_type.unwrap().as_str()), &"a valid asset type"));
      }
    };

    let mut helper = AssetDeserializeHelper::default();
    helper.asset = Some(ass);

    Ok(helper)
  }
}

impl<'de> serde::de::Deserialize<'de> for AssetDeserializeHelper {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where D: serde::de::Deserializer<'de> 
  {
    let ass = deserializer.deserialize_map(AssetDeserializeHelperVisitor { })?;
    Ok(ass) 
  }
}

impl Default for AssetDeserializeHelper {
  fn default() -> Self {
    Self { asset: None }
  }
}

pub struct AssetSerializeHelper<'a, AssetType> {
  asset: &'a AssetType
}

impl<'a, AssetType> AssetSerializeHelper<'a, AssetType> {
  pub fn new(asset: &'a AssetType) -> Self {
    Self { asset }
  }
}

impl<'a, AssetType> serde::Serialize for AssetSerializeHelper<'a, AssetType>
  where AssetType: Asset + serde::Serialize
{
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: serde::Serializer 
  {
    let mut map = serializer.serialize_map(None)?;
    map.serialize_entry("type", self.asset.get_asset_type_name());
    map.serialize_entry("asset", self.asset);
    map.end()
  }
}