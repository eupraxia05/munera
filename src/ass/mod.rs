use std::any::Any;
use std::mem::size_of;
use std::ops::Deref;
use std::{collections::HashMap, fs};
use serde_binary::{Decode, Deserializer};
use serde_binary::binary_stream::Endian;

use crate::math;

pub trait Asset {
  fn decode(&mut self, de: &mut Deserializer) -> serde_binary::Result<()>;
  fn as_any(&self) -> &dyn Any;
  fn size_bytes(&self) -> usize;
}

pub struct ImageAsset {
  pub size: math::Vec2u,
  pub data: Vec<u8>
}

impl ImageAsset {
  fn new() -> Self {
    Self { 
      size: math::Vec2u::new(0, 0),
      data: Vec::<u8>::new()
     }
  }
}

impl Asset for ImageAsset {
  fn decode(&mut self, de: &mut Deserializer) -> serde_binary::Result<()> {
    self.size.x = de.reader.read_u32().expect("Failed to read value!");
    self.size.y = de.reader.read_u32().expect("Failed to read value!");
    self.data = de.reader.read_bytes(self.size.y as usize * self.size.x as usize * size_of::<u8>() as usize * 4).expect("Failed to read data!");
    Ok(())
  }

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn size_bytes(&self) -> usize {
    self.data.len() * size_of::<u8>() + size_of::<math::Vec2u>()
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

  pub fn load_file(&mut self, name: &String) {
    if !self.assets.contains_key(name) {
      log::info!("Loading {}", name);
      if let Ok(read) = fs::read(name) {
        if let Ok(decode) = serde_binary::decode::<AssetDeserializeHelper>(&read, Endian::Little) {
          self.assets.insert(name.clone(), decode.asset.expect("Failed to load asset!"));
        }
        else {
          log::error!("Failed to decode {}", name);
        }
      } else {
        log::error!("Failed to open {}", name);
      }
    }
  }

  pub fn borrow_asset(&self, name: &String) -> Option<&dyn Asset> {
    if let Some(ass) = self.assets.get(name) {
      Some(ass.as_ref())
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
    let ass_type = de.reader.read_string().expect("Couldn't read asset type!");
    let mut asset = match ass_type.as_str() {
      "Image" => Box::new(ImageAsset::new()),
      _ => panic!("Unrecognized asset type: {}", ass_type),
    };
    asset.decode(de).expect("Failed to decode asset!");
    self.asset = Some(asset);
    Ok(())
  }
}

impl Default for AssetDeserializeHelper {
  fn default() -> Self {
    Self { asset: None }
  }
}
