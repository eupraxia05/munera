//use egui::load::SizedTexture;
use egui::{TopBottomPanel, Ui, SidePanel, CentralPanel, Context as EguiContext, ScrollArea, Vec2, Frame, Color32, RichText, TextureId, Stroke, Margin};
use image::{ImageBuffer, EncodableLayout};
use shaderc::ShaderKind;

use crate::engine::{Engine};
use std::io::Write;
use std::cell::RefCell;
use std::fs;
use image::io::Reader as ImageReader;
use image::Rgba;
use std::fs::File;
use crate::{assets, Result, math};

const TOOLBAR_WIDTH: f32 = 32.0f32;
const MIN_CONSOLE_HEIGHT: f32 = 256.0f32;

struct DockTabViewer<'a> {
  asset_cache: &'a RefCell<assets::AssetCache>,
  touched_assets: &'a mut Vec<String>,
}

impl<'a> DockTabViewer<'a> {
  fn new(asset_cache: &'a RefCell<assets::AssetCache>, touched_assets: &'a mut Vec<String>) -> Self {
    Self { asset_cache, touched_assets }
  }
}

impl<'a> egui_dock::TabViewer for DockTabViewer<'a> {
  type Tab = DockTab;

  fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
    match tab {
      DockTab::Asset { name, viewer } => {
        if !self.touched_assets.contains(name) {
          name.as_str().into()
        } else {
          format!("! {}", name).into()
        }
      }
    }
  }
  
  fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
    match tab {
      DockTab::Asset { name, viewer } => {
        let mut cache = self.asset_cache.borrow_mut();
        if let Some(ass) = cache.borrow_asset_generic_mut(name) {
          let modified = viewer.build_dockable_content(ass, ui);
          if modified {
            self.touched_assets.push(name.clone());
          }
        }
      }
    }
  }
}

pub enum DockTab {
  Asset { name: String, viewer: Box<dyn crate::assets::AssetTabViewer> },
}

impl DockTab {
  fn from_asset(name: &String, asset_cache: &std::cell::RefCell<assets::AssetCache>) -> Result<Self> {
    let mut cache = asset_cache.borrow_mut();
    cache.load_file(name)?;
    let ass = cache.borrow_asset_generic_mut(name);
    let viewer = ass.unwrap().create_tab_viewer();
    Ok( Self::Asset { name: name.clone(), viewer } )
  }
}

pub struct Editor<'a, GameAppType>
  where GameAppType: for<'b> crate::engine::App<'b>
{
  phantom_data: std::marker::PhantomData<&'a GameAppType>,
  tools: Vec<Box<dyn Tool>>,
  selected_tool_idx: Option<usize>,
  console_command_input_text: String,
  title_img: egui_extras::RetainedImage,
  dock: egui_dock::DockState<DockTab>,
  touched_assets: Vec<String>,
}

impl<'a, GameAppType> crate::engine::App<'a> for Editor<'a, GameAppType>
  where GameAppType: for<'b> crate::engine::App<'b> + 'static + Default
{
  fn tick(&mut self, dt: f32, device: &wgpu::Device, asset_cache: &RefCell<assets::AssetCache>, egui_rpass: &mut egui_wgpu_backend::RenderPass, queue: &wgpu::Queue) {
    /*for dockable in &mut self.dock.tab {
      dockable.tick(dt, device, asset_cache, egui_rpass, queue);
    }*/
  }

  fn build_ui(&mut self, asset_cache: &RefCell<assets::AssetCache>, egui_context: &egui::Context, device: &wgpu::Device) {
    TopBottomPanel::top("title_menu").show(egui_context, |ui| {
      self.build_title_menu(ui, asset_cache);
    });
    SidePanel::left("toolbar")
      .exact_width(TOOLBAR_WIDTH)
      .resizable(false)
      .show(egui_context, |ui| {
      self.build_toolbar(ui)
    });
    TopBottomPanel::bottom("console")
      .resizable(true)
      .min_height(MIN_CONSOLE_HEIGHT)
      .frame(Frame::default()
        .inner_margin(Margin::same(0.0)))
      .show(egui_context, |ui| 
    {
      self.build_console(ui)
    });
    SidePanel::left("tool_properties").resizable(true).show(egui_context, |ui| {
      self.build_tool_properties(asset_cache, ui, device)
    });

    self.build_dock(asset_cache, egui_context);
  }
}

impl<'a, GameAppType> Default for Editor<'a, GameAppType> 
  where GameAppType: for<'b> crate::engine::App<'b> + 'static + Default
{
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, GameAppType> Editor<'a, GameAppType> 
  where GameAppType: for<'b> crate::engine::App<'b> + 'static + Default
{
  pub fn new() -> Self
  {
    Self {
      phantom_data: Default::default(),
      tools: vec![
        Box::new(AssetBrowserTool::new()), 
        Box::new(PlayTool::<GameAppType>::new()),
        Box::new(AssetCacheTool::new())
      ],
      selected_tool_idx: None,
      dock: egui_dock::DockState::new(Vec::new()),
      console_command_input_text: String::default(),
      title_img: egui_extras::RetainedImage::from_image_bytes("title_img", include_bytes!("../ass/munera.png"))
        .expect("Failed to load title image!"),
      touched_assets: Vec::new(),
   }
  }

  fn build_title_menu(&mut self, ui: &mut Ui, asset_cache: &RefCell<assets::AssetCache>) {
    egui::menu::bar(ui, |ui| {
      ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(self.title_img.texture_id(ui.ctx()), Vec2::new(24.0, 24.0))));
      ui.menu_button("File", |ui| {
        if ui.button("Save All").clicked() {
          for asset in &self.touched_assets {
            let mut cache = asset_cache.borrow_mut();
            let ass = cache.borrow_asset_generic_mut(asset).unwrap().as_any().downcast_ref::<crate::assets::SceneAsset>();
            let str = "t".to_string() + &serde_json::to_string_pretty(&assets::AssetSerializeHelper::new(ass.unwrap())).unwrap();
            std::fs::write(asset, str);
          }
          self.touched_assets.clear();
        }
      });
      ui.menu_button("Project", |ui| {
        let _ = ui.button("Project Settings");
      });
      ui.menu_button("Preferences", |ui| {
        let _ = ui.button("Editor Settings");
      });
      ui.menu_button("Help", |ui| {
        let _ = ui.button("Documentation");
      });
    });
  }

  fn build_toolbar(&mut self, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
      for (idx, tool) in self.tools.iter().enumerate() {
        let button = egui::ImageButton::new(egui::ImageSource::Texture(egui::load::SizedTexture::new(tool.button_img().texture_id(ui.ctx()), Vec2::new(32.0, 32.0))));
        if ui.add(button).clicked() {
          self.selected_tool_idx = Some(idx);
        }
      }
    });
  }

  fn build_tool_properties(&mut self, asset_cache: &RefCell<assets::AssetCache>, ui: &mut Ui, device: &wgpu::Device) {
    ScrollArea::new([false, true]).show(ui, |ui| {
      if self.selected_tool_idx.is_some() {
        let tool = &mut self.tools[self.selected_tool_idx.unwrap()];
        ui.label(tool.name());
        ui.separator();
        tool.build_tool_properties(asset_cache, ui, device, &mut self.dock);
      } else {
        ui.label("Tool Properties");
        ui.separator();
      }
    });
  }

  fn build_console(&mut self, ui: &mut Ui) {
    ui.style_mut().spacing.item_spacing = Vec2::ZERO;
    ui.style_mut().spacing.indent = 0.0;

    TopBottomPanel::top("console_heading_panel").show_inside(ui, |ui| {
      ui.label("Console");
    });

    TopBottomPanel::bottom("console_command_input").show_inside(ui, |ui| {
      let re = ui.text_edit_singleline(&mut self.console_command_input_text);
      if re.lost_focus()  && re.ctx.input(|r| r.key_pressed(egui::Key::Enter)) {
        log::info!("> {}", self.console_command_input_text)
      }
    });

    CentralPanel::default()
      .frame(
        Frame::default()
        .fill(Color32::from_gray(20))
        .stroke(Stroke::new(2.0, Color32::from_gray(50)))
        .inner_margin(Margin::same(4.0))
        .outer_margin(Margin::same(0.0)))
      .show_inside(ui, |ui| 
    {
      ScrollArea::new([false, true])
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| 
      {
        ui.style_mut().spacing.item_spacing = Vec2::new(4.0, 4.0);
        ui.style_mut().spacing.indent = 8.0;

        let messages = crate::logger::LOGGER.messages.lock()
          .expect("Couldn't lock logger messages!");
        for message in messages.clone() {
          let color = match message.level() {
            log::Level::Debug => Color32::GRAY,
            log::Level::Info => Color32::WHITE,
            log::Level::Warn => Color32::YELLOW,
            log::Level::Error => Color32::RED,
            log::Level::Trace => Color32::BROWN
          };
  
          let fmt = format!("{}: {}", message.level(), message.message());
  
          ui.label(RichText::new(fmt).color(color).family(egui::FontFamily::Monospace).size(12.0));
          ui.separator();
        }
      });
    });
  }

  fn build_dock(&mut self, asset_cache: &RefCell<assets::AssetCache>, ctx: &EguiContext) {
    egui::CentralPanel::default().show(ctx, |ui| {
      let mut viewer = DockTabViewer::new(asset_cache, &mut self.touched_assets);
      egui_dock::DockArea::new(&mut self.dock).style(egui_dock::Style::from_egui(ctx.style().as_ref()))
        .show_inside(ui, &mut viewer);
    });
    
    /*TopBottomPanel::top("dock_tabs").show(ctx, |ui| {
      ui.horizontal(|ui| {
        let mut close_idx = None;
        for (idx, dockable) in self.dock.dockables.iter().enumerate() {
          let tab = ui.button(dockable.title());
          
          if tab.clicked() {
            self.dock.focused_dockable = idx;
          } else if tab.secondary_clicked() {
            close_idx = Some(idx);
          }
        }

        if let Some(idx) = close_idx {
          self.dock.dockables.remove(idx);
        }
      })
    });
    CentralPanel::default().show(ctx, |ui| {
      if self.dock.dockables.len() > self.dock.focused_dockable {
        self.dock.dockables[self.dock.focused_dockable].build_content(asset_cache, ui);
      }
    });*/
  }
}

trait Tool {
  fn name(&self) -> &'static str;
  fn button_img(&self) -> &egui_extras::RetainedImage;
  fn build_tool_properties(&mut self, asset_cache: &RefCell<assets::AssetCache>, ui: &mut Ui, device: &wgpu::Device, dock: &mut egui_dock::DockState<DockTab>);
}

struct AssetBrowserTool {
  import_path: String,
  target_path: String,
  import_handlers: Vec<Box<dyn ImportHandler>>,
  selected_asset: Option<String>,
  button_img: egui_extras::RetainedImage,
  new_asset_name: String
}

impl AssetBrowserTool {
  fn new() -> Self {
    Self {
      import_path: String::from(""),
      target_path: String::from(""),
      import_handlers: vec![Box::new(ImageImportHandler::new()), 
        Box::new(ShaderImportHandler::new())],
      selected_asset: None,
      button_img: egui_extras::RetainedImage::from_image_bytes("asset_browser_tool_button", 
        include_bytes!("../ass/asset_browser.png")).expect("Failed to load image!"),
      new_asset_name: String::from(""),
    }
  }
}

impl Tool for AssetBrowserTool {
  fn name(&self) -> &'static str {
    "Asset Browser"
  }

  fn button_img(&self) -> &egui_extras::RetainedImage {
    &self.button_img
  }

  fn build_tool_properties(&mut self, asset_cache: &RefCell<assets::AssetCache>, ui: &mut Ui, device: &wgpu::Device, dock: &mut egui_dock::DockState<DockTab>) {
    ui.collapsing("New", |ui| {
      ui.horizontal(|ui| {
        ui.label("Name");
        ui.separator();
        ui.text_edit_singleline(&mut self.new_asset_name);
      });

      if ui.button("Scene").clicked() {
        let scene = crate::assets::SceneAsset::default();
        let ser = "t".to_string() + &serde_json::to_string_pretty(&assets::AssetSerializeHelper::new(&scene)).expect("Couldn't serialize default scene!");
        std::fs::write(format!("ass/{}.ass", self.new_asset_name), ser).expect("Couldn't write to file!");
      }
    });
    ui.collapsing("Import", |ui| {
      ui.horizontal(|ui| {
        ui.label("Source");
        ui.separator();
        ui.label(self.import_path.clone());
        if ui.button("Browse").clicked() {
          let result = tinyfiledialogs::open_file_dialog("Browse for Import Source", "./", None);
          if result.is_some() {
            self.import_path = result.unwrap();
          }
        }
      });

      ui.horizontal(|ui| {
        ui.label("Target Dir");
        ui.separator();
        ui.label(self.target_path.clone());
        if ui.button("Browse").clicked() {
          let result = tinyfiledialogs::save_file_dialog_with_filter("Browse for Import Target", "./", &["*.ass"]
            , "Munera Asset File (.ass)");
          if result.is_some() {
            self.target_path = result.unwrap();
          }
        }
      });

      let mut ass_type = None;
      'handler_loop: for (idx, handler) 
        in self.import_handlers.iter().enumerate() 
      {
        for extension in handler.extensions() {
          if self.import_path.ends_with(extension) {
            ass_type = Some(idx);
            break 'handler_loop
          }
        }
      }

      if ass_type.is_some() {
        ui.label(self.import_handlers[ass_type.unwrap()].name());
      } else {
        ui.label("Unsupported");
      }

      if ui.button("Import").clicked() && ass_type.is_some() {
        let handler = &self.import_handlers[ass_type.unwrap()];
        log::info!("Importing: [{}] to asset file: [{}] Asset type: {}",
          self.import_path, self.target_path, handler.name());
        handler.import(&self.import_path, &self.target_path);
      }
    });

    let paths = fs::read_dir("./ass/").unwrap();

    for path in paths {
      let p = path.unwrap().file_name();
      let name = p.to_str().unwrap();
      let is_selected = self.selected_asset.is_some() && name == self.selected_asset.clone().unwrap();
      if ui.selectable_label(is_selected, name).clicked() {
        let file_path = String::from("./ass/") + &String::from(name);
        match DockTab::from_asset(&file_path, asset_cache) {
          Ok(tab) => {
            dock.push_to_focused_leaf(tab);
          },
          Err(err) => {
            log::error!("Couldn't open asset tab: {}", err)
          }
        }
      }
    }
  }
}

struct PlayTool<GameAppType>
  where GameAppType : for<'a> crate::engine::App<'a> + Default
{
  button_img: egui_extras::RetainedImage,
  phantom_data: std::marker::PhantomData<GameAppType>
}

impl<GameAppType> Tool for PlayTool<GameAppType>
  where GameAppType : for<'a> crate::engine::App<'a> + Default + 'static
{
  fn name(&self) -> &'static str {
    "Play"
  }

  fn button_img(&self) -> &egui_extras::RetainedImage {
    &self.button_img
  }

  fn build_tool_properties(&mut self, _asset_cache: &RefCell<assets::AssetCache>, ui: &mut Ui, device: &wgpu::Device, dock: &mut egui_dock::DockState<DockTab>) {
    if ui.button("Play!").clicked() {
      //dock.dockables.push(Box::new(PlayDockable::<GameAppType>::new(device)));
    }
  }
}

impl<GameAppType> PlayTool<GameAppType>
  where GameAppType : for<'a> crate::engine::App<'a> + Default
{
  fn new() -> Self {
    Self {
      button_img: egui_extras::RetainedImage::from_image_bytes("play_tool_button", 
        include_bytes!("../ass/play.png")).expect("Failed to load image!"),
      phantom_data: std::marker::PhantomData::default()
    }
  }
}

fn to_mem_size_str(size: usize) -> String {
  let postfixes = &["B", "KiB", "MiB", "GiB", "TiB"];
  let mut curr_size = size;
  for postfix in postfixes {
    if curr_size < 1024 {
      return format!("{} {}", curr_size, postfix)
    }
    curr_size /= 1024;
  }
  format!("{} B", size)
}

struct AssetCacheTool {
  button_img: egui_extras::RetainedImage
}

impl AssetCacheTool {
  fn new() -> Self {
    Self {
      button_img: egui_extras::RetainedImage::from_image_bytes("asset_cache_tool_button", 
        include_bytes!("../ass/registry_editor.png")).expect("Failed to load image!")
    }
  }
}

impl Tool for AssetCacheTool {
  fn name(&self) -> &'static str {
    "Asset Cache"
  }

  fn button_img(&self) -> &egui_extras::RetainedImage {
    &self.button_img
  }

  fn build_tool_properties(&mut self, asset_cache: &RefCell<assets::AssetCache>, ui: &mut Ui, device: &wgpu::Device, dock: &mut egui_dock::DockState<DockTab>) 
  {
    let cache = asset_cache.borrow();
    let assets = cache.borrow_all_assets();
    let mut size = 0;
    assets.iter().for_each(|ass| {
      ui.label(ass.0);
      size += ass.1.size_bytes();
    });
    ui.separator();
    ui.label(format!("Total asset memory: {}", to_mem_size_str(size)));
  }
}

/*pub trait Dockable {
  fn title(&self) -> String;
  fn build_content(&mut self, asset_cache: &RefCell<assets::AssetCache>, ui: &mut Ui);
  fn tick(&mut self, dt: f32, device: &wgpu::Device, asset_cache: &RefCell<assets::AssetCache>, egui_rpass: &mut egui_wgpu_backend::RenderPass, queue: &wgpu::Queue) { }
}

struct AssetEditorDockable {
  ass_name: String
}

impl AssetEditorDockable {
  fn new(asset_cache: &RefCell<assets::AssetCache>, path: &String) -> Result<Self> {
    let mut cache = asset_cache.borrow_mut();
    cache.load_file(path)?;
    Ok( Self { ass_name: path.clone() } )
  }
}

impl Dockable for AssetEditorDockable {
  fn title(&self) -> String {
    self.ass_name.clone()
  }

  fn build_content(&mut self, asset_cache: &RefCell<assets::AssetCache>, ui: &mut Ui) {
    let mut ass_cache = asset_cache.borrow_mut();
    let ass = ass_cache.borrow_asset_generic_mut(&self.ass_name);
    if ass.is_some() {
      ass.unwrap().build_dockable_content(ui);
    }
  }
}

struct PlayDockable<GameAppType> {
  requested_size: math::Vec2u,
  curr_size: math::Vec2u,
  image: Option<wgpu::Texture>,
  tex_id: Option<egui::TextureId>,
  game_app: GameAppType,
  egui_rpass: egui_wgpu_backend::RenderPass,
  egui_ctx: egui::Context
}

impl<GameAppType> PlayDockable<GameAppType>
  where GameAppType: crate::engine::App + Default  
{
  fn new(device: &wgpu::Device) -> Self {
    Self { 
      image: None, 
      requested_size: math::Vec2u::new(0, 0), 
      curr_size: math::Vec2u::new(0, 0), 
      tex_id: None,
      game_app: GameAppType::default(),
      egui_rpass: egui_wgpu_backend::RenderPass::new(device, wgpu::TextureFormat::Rgba16Float, 1),
      egui_ctx: egui::Context::default()
    }
  }

  fn update_img(&mut self, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass) {
    if self.requested_size.x == 0 || self.requested_size.y == 0 {
      return;
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

    if self.image.is_some() {
      if self.requested_size != self.curr_size {
        let mut delta = egui::TexturesDelta::default();
        delta.free.push(self.tex_id.unwrap());
        egui_rpass.remove_textures(delta);
        self.image.as_mut().unwrap().destroy();
      } else {
        return;
      }
    }

    let image = device.create_texture(&tex_desc);
    log::info!("Creating play texture {} x {}", self.requested_size.x, self.requested_size.y);

    self.tex_id = Some(egui_rpass.egui_texture_from_wgpu_texture(device, 
      &image.create_view(&wgpu::TextureViewDescriptor::default()), 
      wgpu::FilterMode::Nearest));

    self.curr_size = self.requested_size;
    self.image = Some(image);
  }
}

impl<GameAppType> Dockable for PlayDockable<GameAppType>
  where GameAppType: crate::engine::App + Default
{
  fn title(&self) -> String {
    String::from("Play")
  }

  fn tick(&mut self, dt: f32, device: &wgpu::Device, asset_cache: &RefCell<assets::AssetCache>, egui_rpass: &mut egui_wgpu_backend::RenderPass, queue: &wgpu::Queue) {
    self.update_img(device, egui_rpass);

    if self.image.is_some() {
      let tex_view = self.image.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor::default());
      let mut input = egui::RawInput::default();
      input.screen_rect = Some(egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), 
        egui::Pos2::new(self.curr_size.x as f32, self.curr_size.y as f32)));
      self.egui_ctx.begin_frame(input);
      self.game_app.build_ui(asset_cache, &self.egui_ctx, device);
      let result = self.egui_ctx.end_frame();
      let paint_jobs = self.egui_ctx.tessellate(result.shapes);

      let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("PlayDockable render encoder")
      });

      let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
        physical_width: self.image.as_ref().unwrap().width(),
        physical_height: self.image.as_ref().unwrap().height(),
        scale_factor: 1.0
      };

      self.egui_rpass.add_textures(device, queue, &result.textures_delta);
      self.egui_rpass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);
      self.egui_rpass.execute(&mut encoder, &tex_view, &paint_jobs, &screen_descriptor, Some(wgpu::Color::BLACK));
      queue.submit(std::iter::once(encoder.finish()));
    }
  }

  fn build_content(&mut self, _asset_cache: &RefCell<assets::AssetCache>, ui: &mut Ui) {
    let size = ui.available_size();
    self.requested_size = math::Vec2u::new(size.x as u32, size.y as u32);
    
    if self.tex_id.is_some() {
      ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(self.tex_id.unwrap(), Vec2::new(self.curr_size.x as f32, self.curr_size.y as f32))));

    }
  }
}*/

trait ImportHandler {
  fn name(&self) -> &'static str;
  fn extensions(&self) -> &[&'static str];
  fn import(&self, import_path: &String, target_path: &String);
}

struct ImageImportHandler;

impl ImageImportHandler {
  fn new() -> Self {
    Self { }
  }
}

struct ImageImportSerializeHelper<'a> {
  img: &'a ImageBuffer<Rgba<u8>, Vec<u8>>
}

impl<'a> serde_binary::Encode for ImageImportSerializeHelper<'a> {
  fn encode(&self, ser: &mut serde_binary::Serializer) -> serde_binary::Result<()> {
    ser.writer.write_string("Image")?;
    ser.writer.write_u32(self.img.width())?;
    ser.writer.write_u32(self.img.height())?;
    ser.writer.write_bytes(self.img.as_bytes())?;
    Ok(())
  }
}

impl ImportHandler for ImageImportHandler {
  fn name(&self) -> &'static str {
    "Image"
  }

  fn extensions(&self) -> &[&'static str] {
    &[".png", ".jpg", ".bmp", ".tga", ".exr", ".hdr"]
  }

  fn import(&self, import_path: &String, target_path: &String) {
    let open = ImageReader::open(import_path);
    if open.is_ok() {
      let decode = open.unwrap().decode();
      if decode.is_ok() {
        let img = decode.unwrap();
        let rgba = img.into_rgba8();
        let enc = serde_binary::encode(&ImageImportSerializeHelper{img: &rgba}, 
          serde_binary::binary_stream::Endian::Little);
        let file = File::create(target_path);
        if file.is_ok() {
          if enc.is_ok() {
            let mut buf_to_write = vec![b'b'];
            buf_to_write.append(&mut enc.unwrap().clone());
            let write_result = file.unwrap().write_all(&buf_to_write);
            if write_result.is_err() {
              log::error!("Failed to write {}: {}", target_path, write_result.err().unwrap())
            }
          } else {
            log::error!("Failed to encode {}: {}", import_path, enc.err().unwrap());
          }
        } 
        else {
          log::error!("Failed to open {}", target_path);
        }
      } else {
        log::error!("Failed to decode {}", import_path)
      }
    } else {
      log::error!("Failed to open {}", import_path);
    }
  } 
}

struct ShaderImportHandler;

impl ShaderImportHandler {
  fn new() -> Self { 
    Self { }
  }
}

impl ImportHandler for ShaderImportHandler {
  fn name(&self) -> &'static str {
    "Shader"
  }

  fn extensions(&self) -> &[&'static str] {
    &[".vert", ".frag"]
  }

  fn import(&self, import_path: &String, target_path: &String) {
    let shader_kind = if import_path.ends_with(".vert") {
      shaderc::ShaderKind::Vertex
    } else if import_path.ends_with(".frag") {
      shaderc::ShaderKind::Fragment
    } else {
      log::error!("Invalid shader file extension: {}", import_path);
      return
    };

    match fs::read_to_string(import_path) {
      Err(err) => {
        log::error!("Failed to read {}: {}", import_path, err.to_string())
      },
      Ok(code) => {
        if let Some(compiler) = shaderc::Compiler::new() {
          match compiler.compile_into_spirv(&code, shader_kind, 
            import_path, "main", None) 
          {
            Err(err) => {
              log::error!("Failed to compile {}: {}", import_path, err.to_string());
            },
            Ok(spirv) => {
              let mut ass = assets::ShaderAsset::default();
              ass.shader_type = match shader_kind {
                ShaderKind::Vertex => assets::ShaderType::Vertex,
                ShaderKind::Fragment => assets::ShaderType::Fragment,
                _ => assets::ShaderType::Vertex
              };
              ass.code = spirv.as_binary_u8().to_vec();
              match serde_binary::encode(&ass, 
                serde_binary::binary_stream::Endian::Little) 
              {                
                Err(err) => {
                  log::error!("Failed to encode shader: {}", err);
                },
                Ok(mut bin) => {
                  let mut final_out = vec![b'b'];
                  final_out.append(&mut bin);
                  match fs::write(target_path, final_out) {
                    Err(err) => {
                      log::error!("Failed to write to {}: {}", target_path, err.to_string());
                    }
                    Ok(()) => {
                      
                    }
                  }
                }
              }
            }
          }
        } else {
          log::error!("Failed to create SPIRV compiler!");
        }
      }
    }
  }
}
