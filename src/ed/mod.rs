//use egui::load::SizedTexture;
use egui::{TopBottomPanel, Ui, SidePanel, CentralPanel, Context as EguiContext, ScrollArea, Vec2, Frame, Color32, RichText, TextureId, Stroke, Margin};
use image::{ImageBuffer, EncodableLayout};
use shaderc::ShaderKind;

use crate::ass::{AssetCache, ShaderAsset, ShaderType};
use crate::engine::{Engine, eng_log};
use std::io::Write;
use std::cell::RefCell;
use std::fs;
use image::io::Reader as ImageReader;
use image::Rgba;
use std::fs::File;
use crate::Result;

const TOOLBAR_WIDTH: f32 = 64.0f32;
const MIN_CONSOLE_HEIGHT: f32 = 256.0f32;

pub struct Dock {
  dockables: Vec<Box<dyn Dockable>>,
  focused_dockable: usize
}

impl Dock {
  pub fn new() -> Self {
    Self {
      dockables: Vec::new(),
      focused_dockable: 0
    }
  }
}

pub struct Editor {
  tools: Vec<Box<dyn Tool>>,
  selected_tool_idx: Option<usize>,
  dock: Dock,
  console_command_input_text: String,
  title_img: egui_extras::RetainedImage
}

impl crate::engine::App for Editor {
  fn build_ui(&mut self, asset_cache: &RefCell<crate::ass::AssetCache>, egui_context: &egui::Context) {
    TopBottomPanel::top("title_menu").show(egui_context, |ui| {
      self.build_title_menu(ui);
    });
    SidePanel::left("toolbar").exact_width(TOOLBAR_WIDTH)
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
      self.build_tool_properties(asset_cache, ui)
    });

    self.build_dock(asset_cache, egui_context);
  }
}

impl Editor {
  pub fn new() -> Self {
    Self {
      tools: vec![
        Box::new(AssetBrowserTool::new()), 
        Box::new(PlayTool{ }),
        Box::new(AssetCacheTool{ })],
      selected_tool_idx: None,
      dock: Dock::new(),
      console_command_input_text: String::default(),
      title_img: egui_extras::RetainedImage::from_image_bytes("title_img", include_bytes!("../../ass/tuwuck.png"))
        .expect("Failed to load title image!")
   }
  }

  fn build_title_menu(&self, ui: &mut Ui) {
    egui::menu::bar(ui, |ui| {
      ui.image(self.title_img.texture_id(ui.ctx()), Vec2::new(16.0, 16.0));
      ui.menu_button("File", |ui| {
        ui.menu_button("New...", |ui| {
          let _ = ui.button("Scene");
        });
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
    ui.label("Toolbar");
    ui.separator();
    for (idx, tool) in self.tools.iter().enumerate() {
      if ui.button(tool.name()).clicked() {
        self.selected_tool_idx = Some(idx);
      }
    }
  }

  fn build_tool_properties(&mut self, asset_cache: &RefCell<AssetCache>, ui: &mut Ui) {
    ScrollArea::new([false, true]).show(ui, |ui| {
      if self.selected_tool_idx.is_some() {
        let tool = &mut self.tools[self.selected_tool_idx.unwrap()];
        ui.label(tool.name());
        ui.separator();
        tool.build_tool_properties(asset_cache, &mut self.dock, ui);
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

        let messages = eng_log::LOGGER.messages.lock()
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

  fn build_dock(&mut self, asset_cache: &RefCell<AssetCache>, ctx: &EguiContext) {
    TopBottomPanel::top("dock_tabs").show(ctx, |ui| {
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
    });
  }
}

trait Tool {
  fn name(&self) -> &'static str;
  fn build_tool_properties(&mut self, asset_cache: &RefCell<AssetCache>, dock: &mut Dock, ui: &mut Ui);
}

struct AssetBrowserTool {
  import_path: String,
  target_path: String,
  import_handlers: Vec<Box<dyn ImportHandler>>,
  selected_asset: Option<String>,
}

impl AssetBrowserTool {
  fn new() -> Self {
    Self {
      import_path: String::from(""),
      target_path: String::from(""),
      import_handlers: vec![Box::new(ImageImportHandler::new()), 
        Box::new(MeshImportHandler::new()), Box::new(ShaderImportHandler::new())],
      selected_asset: None,
    }
  }
}

impl Tool for AssetBrowserTool {
  fn name(&self) -> &'static str {
    "Asset Browser"
  }

  fn build_tool_properties(&mut self, asset_cache: &RefCell<AssetCache>, dock: &mut Dock, ui: &mut Ui) {
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
        match AssetEditorDockable::new(asset_cache,
          &(String::from("./ass/") + &String::from(name))) 
        {
          Err(e) => {
            log::error!("Failed to open {}: {}", name, e);
          },
          Ok(dockable) => {
            dock.dockables.push(Box::new(dockable));
          }
        }
      }
    }
  }
}

struct PlayTool;

impl Tool for PlayTool {
  fn name(&self) -> &'static str {
    "Play"
  }

  fn build_tool_properties(&mut self, _asset_cache: &RefCell<AssetCache>, dock: &mut Dock, ui: &mut Ui) {
    if ui.button("Play!").clicked() {
      dock.dockables.push(Box::new(PlayDockable { }));
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

struct AssetCacheTool;

impl Tool for AssetCacheTool {
  fn name(&self) -> &'static str {
    "Asset Cache"
  }

  fn build_tool_properties(&mut self, asset_cache: &RefCell<AssetCache>, 
    _dock: &mut Dock, ui: &mut Ui) 
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

pub trait Dockable {
  fn title(&self) -> String;
  fn build_content(&self, asset_cache: &RefCell<AssetCache>, ui: &mut Ui);
}

struct AssetEditorDockable {
  ass_name: String
}

impl AssetEditorDockable {
  fn new(asset_cache: &RefCell<AssetCache>, path: &String) -> Result<Self> {
    let mut cache = asset_cache.borrow_mut();
    cache.load_file(path)?;
    Ok( Self { ass_name: path.clone() } )
  }
}

impl Dockable for AssetEditorDockable {
  fn title(&self) -> String {
    self.ass_name.clone()
  }

  fn build_content(&self, asset_cache: &RefCell<AssetCache>, ui: &mut Ui) {
    let mut ass_cache = asset_cache.borrow_mut();
    let ass = ass_cache.borrow_asset_mut(&self.ass_name);
    if ass.is_some() {
      ass.unwrap().build_dockable_content(ui);
    }
  }
}

struct PlayDockable;

impl Dockable for PlayDockable {
  fn title(&self) -> String {
    String::from("Play")
  }

  fn build_content(&self, _asset_cache: &RefCell<AssetCache>, ui: &mut Ui) {
    //ui.image(SizedTexture::new(screen_tex, Vec2::new(200.0, 200.0)));
  }
}

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
            let write_result = file.unwrap().write_all(&enc.unwrap());
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

struct MeshImportHandler;

impl MeshImportHandler {
  fn new() -> Self {
    Self { }
  }
}

impl ImportHandler for MeshImportHandler {
  fn name(&self) -> &'static str {
    "Mesh"
  }

  fn extensions(&self) -> &[&'static str] {
    &[".gltf", ".fbx", ".obj"]
  }

  fn import(&self, _import_path: &String, _target_path: &String) {

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
              let mut ass = ShaderAsset::default();
              ass.shader_type = match shader_kind {
                ShaderKind::Vertex => ShaderType::Vertex,
                ShaderKind::Fragment => ShaderType::Fragment,
                _ => ShaderType::Vertex
              };
              ass.code = spirv.as_binary_u8().to_vec();
              match serde_binary::encode(&ass, 
                serde_binary::binary_stream::Endian::Little) 
              {                
                Err(err) => {
                  log::error!("Failed to encode shader: {}", err);
                },
                Ok(bin) => {
                  match fs::write(target_path, bin) {
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
