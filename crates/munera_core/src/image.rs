#[derive(munera_macros::Asset, serde::Serialize, serde::Deserialize, Default)]
pub struct ImageAsset {
  pub size: munera_math::Vec2u,
  pub data: Vec<u8>,
  name: String
}

impl ImageAsset {
  fn new() -> Self {
    Self { 
      size: munera_math::Vec2u::new(0, 0),
      data: Vec::<u8>::new(),
      name: String::new(),
    }
  }
}

impl munera_assets::AssetCreateTabViewer for ImageAsset {  
  fn create_tab_viewer(&self) -> Box<dyn munera_assets::AssetTabViewer> {
    Box::new(ImageAssetTabViewer::new())
  }
}

impl munera_assets::AssetPostLoad for ImageAsset {
  fn post_load(&mut self, name: &str) {
    self.name = name.to_string();
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
  fn decode(&mut self, de: &mut serde_binary::Deserializer) -> serde_binary::Result<()> {
    self.size.x = de.reader.read_u32().expect("Failed to read value!");
    self.size.y = de.reader.read_u32().expect("Failed to read value!");
    self.data = de.reader.read_bytes(self.size.y as usize * self.size.x as usize
      * std::mem::size_of::<u8>() as usize * 4).expect("Failed to read data!");
    
    Ok(())
  }
}

struct ImageAssetTabViewer {
  zoom: f32,
  retained_image: Option<egui_extras::RetainedImage>
}

impl ImageAssetTabViewer {
  fn new() -> Self {
    Self { 
      zoom: 1.0, 
      retained_image: None 
    }
  }
}

impl munera_assets::AssetTabViewer for ImageAssetTabViewer {
  fn tick(&mut self, asset: &mut dyn munera_assets::Asset, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass, 
    output_tex_view: &wgpu::TextureView, queue: &wgpu::Queue) 
  {
      
  }

  fn build_dockable_content(&mut self, asset: &mut dyn munera_assets::Asset, ui: &mut egui::Ui) -> bool {
    if let Some(img) = asset.as_any().downcast_ref::<ImageAsset>() {
      if self.retained_image.is_none() {
        let col_img = 
          egui::ColorImage::from_rgba_premultiplied(
            [img.size.x as usize, img.size.y as usize], 
            &img.data);

        self.retained_image = Some(egui_extras::RetainedImage::from_color_image(img.name.clone(), col_img))
      }

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
          ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(self.retained_image.as_ref().unwrap()
            .texture_id(ui.ctx()), egui::Vec2::new(disp_w as f32, disp_h as f32))));
        });
      });
    } else {
      log::error!("ImageAssetTabViewer was used for an asset that wasn't an image!");
    }

    false
  }
}