use std::fs;

fn main() {
  let mut cache = munera_assets::AssetCache::new();
  cache.set_base_path(&"./assets".to_string());
  std::fs::write("./assets.blob", cache.encode_blob());
}