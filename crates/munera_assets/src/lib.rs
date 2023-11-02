use std::any::Any;
use std::mem::size_of;
use std::{collections::HashMap, fs};
use egui_extras::RetainedImage;
use serde::ser::SerializeMap;
use serde_binary::{Decode, Deserializer};
use serde_binary::binary_stream::Endian;
use spirv_reflect::ShaderModule;

/// A standard Result type used by various engine systems.
pub type Result<T> = std::result::Result<T, Error>;

/// A standard Error type used by various engine systems.
#[derive(Debug, Clone)]
pub struct Error {
  message: String
}

impl Error {
  fn new<T>(msg: &T) -> Self where T: ToString + ?Sized {
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

pub trait Asset : serde_binary::Encode + serde_binary::Decode + Any 
  + AssetPostLoad + AssetCreateTabViewer
{
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait AssetExt : Asset + Default 
  + for<'de> serde::Deserialize<'de> + serde::Serialize 
{
  fn asset_type_name() -> &'static str;
}

pub trait AssetPostLoad {
  fn post_load(&mut self, name: &str);
}

pub trait AssetCreateTabViewer {
  fn create_tab_viewer(&self) -> Box<dyn AssetTabViewer>;
}

#[derive(Clone)]
pub struct AssetType {
  type_id: std::any::TypeId,
  name: &'static str,
  create_default_fn: fn() -> Box<dyn Asset>,
  from_value_fn: fn(serde_json::Value) -> Box<dyn Asset>,
}

impl AssetType {
  pub fn new<T>() -> Self 
    where T: AssetExt
  {
    Self { 
      type_id: std::any::TypeId::of::<T>(),
      name: T::asset_type_name(),
      create_default_fn: Self::impl_create_default_fn::<T>,
      from_value_fn: Self::impl_from_value_fn::<T>
    }
  }

  fn find<T>() -> Option<Self> 
    where T: AssetExt
  {
    for asset_type in inventory::iter::<AssetType>() {
      if asset_type.type_id == std::any::TypeId::of::<T>() {
        return Some(asset_type.clone());
      }
    }

    None
  }

  fn find_by_name(name: &str) -> Option<Self> {
    for asset_type in inventory::iter::<AssetType>() {
      if asset_type.name == name {
        return Some(asset_type.clone());
      }
    }
    
    None
  }

  fn impl_create_default_fn<T>() -> Box<dyn Asset>
    where T: AssetExt
  {
    Box::new(T::default())
  }

  fn impl_from_value_fn<T>(value: serde_json::Value) -> Box<dyn Asset> 
    where T: AssetExt
  {
    Box::new(serde_json::from_value::<T>(value).unwrap())
  }
}

inventory::collect!(AssetType);

pub trait AssetTabViewer {
  fn tick(&mut self, asset: &mut dyn Asset, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass, 
    output_tex_view: &wgpu::TextureView, queue: &wgpu::Queue);
  fn build_dockable_content(&mut self, asset: &mut dyn Asset, ui: &mut egui::Ui) -> bool;
}

/*#[derive(serde_repr::Serialize_repr, Clone, Copy, Debug)]
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
}*/

pub struct AssetCache {
  base_path: String,
  assets: HashMap<String, Box<dyn Asset>>
}

impl AssetCache {
  pub fn new() -> Self {
    Self {
      base_path: String::new(),
      assets: HashMap::new()
    }
  }

  pub fn set_base_path(&mut self, base_path: &String) {
    self.base_path = base_path.clone();

    log::info!("Setting asset base path to {}", self.base_path);

    if let Ok(paths) = std::fs::read_dir(base_path) {
      for path in paths {
        let p = path.unwrap().file_name();
        let name = p.to_str().unwrap().to_string();
        if let Err(err) = self.load_file(&name) {
          log::error!("Could not load {}: {}", name, err);
        }
      }
    }
  }

  pub fn load_file(&mut self, name: &String) -> Result<()> {
    if !self.assets.contains_key(name) {
      let path = self.base_path.clone() + "/" + name;
      log::info!("Loading {}", path);
      if let Ok(read) = fs::read(path.clone()) {
        if read[0] == b'b' {
          match serde_binary::decode::<AssetDeserializeHelper>(&read[1..], Endian::Little) {
            Ok(decode) => {
              let mut ass = decode.asset.unwrap();
              ass.post_load(name);
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
              ass.post_load(name);
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
        return Err(Error::new(&format!("Failed to open {}", path)));
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
      let mut asset = match AssetType::find_by_name(&ass_type) {
        Some(asset_type) => {
          (asset_type.create_default_fn)()
        },
        None => {
          return Err(serde_binary::Error::Custom(
            format!("Unrecognized asset type: {}", ass_type)));
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

    let ass = match AssetType::find_by_name(
      asset_type.clone().unwrap().as_str()) 
    {
      Some(asset_type) => {
        (asset_type.from_value_fn)(asset_value.unwrap().clone())
      },
      None => {
        return Err(serde::de::Error::invalid_value(
          serde::de::Unexpected::Str(format!("Unrecognized asset type: {}", 
          asset_type.unwrap().as_str()).as_str()), &"a valid asset type"));
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
  where AssetType: AssetExt
{
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: serde::Serializer 
  {
    let mut map = serializer.serialize_map(None)?;
    map.serialize_entry("type", AssetType::asset_type_name());
    map.serialize_entry("asset", self.asset);
    map.end()
  }
}