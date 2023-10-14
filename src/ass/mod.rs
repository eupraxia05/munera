use std::any::Any;
use std::mem::size_of;
use std::ops::Deref;
use std::{collections::HashMap, fs};
//use egui::load::SizedTexture;
use egui_extras::RetainedImage;
use serde_binary::{Decode, Deserializer};
use serde_binary::binary_stream::Endian;
use serde::{Serialize, Deserialize};
use spirv_reflect::ShaderModule;

use crate::math;
use crate::Result;
use crate::Error;

pub trait Asset : serde_binary::Encode + serde_binary::Decode {
  fn as_any(&self) -> &dyn Any;
  fn size_bytes(&self) -> usize;
  fn build_dockable_content(&mut self, ui: &mut egui::Ui);
  fn set_name(&mut self, name: &String);
  fn post_load(&mut self);
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
}

impl Asset for ImageAsset {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn size_bytes(&self) -> usize {
    self.data.len() * size_of::<u8>() + size_of::<math::Vec2u>()
  }

  fn build_dockable_content(&mut self, ui: &mut egui::Ui) {
    egui::SidePanel::new(egui::panel::Side::Right, egui::Id::new("AssetEditorDockable")).show(ui.ctx(), |ui| {
      ui.label(format!("Resolution: {} x {}", self.size.x, self.size.y));
    });
    egui::CentralPanel::default().show(ui.ctx(), |ui| {
      ui.centered_and_justified(|ui| {
        let w = self.size.x;
        let h = self.size.y;
        let aspect = w as f32 / h as f32;
        let disp_h = std::cmp::min(ui.available_height() as u32, h);
        let disp_w = std::cmp::min(ui.available_width() as u32, 
          (disp_h as f32 * aspect) as u32);
        let disp_h = std::cmp::min(ui.available_height() as u32, 
          (disp_w as f32 / aspect) as u32);
        ui.image(self.retained_image.as_ref().unwrap()
          .texture_id(ui.ctx()), egui::Vec2::new(disp_w as f32, disp_h as f32));
      });
    });
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
  fn encode(&self, ser: &mut serde_binary::Serializer) 
    -> serde_binary::Result<()> 
  {
    Ok(())
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
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn size_bytes(&self) -> usize {
    size_of::<Self>() + self.code.len() * size_of::<u8>()
  }

  fn build_dockable_content(&mut self, ui: &mut egui::Ui) {
    ui.label(format!("Type: {}", self.shader_type.to_string()));

    let module = self.shader_module.as_ref().expect("Null shader module!");
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
        match serde_binary::decode::<AssetDeserializeHelper>(&read, Endian::Little) {
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
      } else {
        return Err(Error::new(&format!("Failed to open {}", name)));
      }
    }
    Ok(())
  }

  pub fn borrow_asset(&self, name: &String) -> Option<&dyn Asset> {
    if let Some(ass) = self.assets.get(name) {
      Some(ass.as_ref())
    } else {
      None
    }
  }

  pub fn borrow_asset_mut(&mut self, name: &String) -> Option<&mut dyn Asset> {
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

impl Default for AssetDeserializeHelper {
  fn default() -> Self {
    Self { asset: None }
  }
}
