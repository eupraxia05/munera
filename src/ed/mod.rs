use egui::{TopBottomPanel, Ui, SidePanel, CentralPanel, Context as EguiContext, ScrollArea, TextureHandle, ColorImage, Vec2, Frame, Color32, RichText, Style};
use image::{ImageBuffer, EncodableLayout};

use crate::ass::{AssetCache, ImageAsset};
use crate::eng::{Engine, eng_log};
use std::io::Write;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use egui_extras::RetainedImage;
use std::fs;
use image::io::Reader as ImageReader;
use image::Rgba;
use std::fs::File;
use std::cmp;

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
  blass: RetainedImage,
  tools: Vec<Box<dyn Tool>>,
  selected_tool_idx: Option<usize>,
  dock: Dock
}

impl Editor {
  pub fn new() -> Self {
    Self {
      blass: RetainedImage::from_image_bytes("tuwuck.png", include_bytes!("../../ass/tuwuck.png"))
        .expect("Failed to load image!"),
      tools: vec![
        Box::new(AssetBrowserTool::new()), 
        Box::new(PlayTool{ }),
        Box::new(AssetCacheTool{ })],
      selected_tool_idx: None,
      dock: Dock::new()
   }
  }

  pub fn run(&mut self, engine: &mut Engine) {
    'main_loop : loop {
      let fun = |ctx: &EguiContext| {
        TopBottomPanel::top("title_menu").show(&ctx, |ui| {
          self.build_title_menu(&ctx, ui);
        });
        SidePanel::left("toolbar").exact_width(TOOLBAR_WIDTH)
          .show(&ctx, |ui| {
          self.build_toolbar(ui)
        });
        TopBottomPanel::bottom("console").resizable(true).min_height(MIN_CONSOLE_HEIGHT).show(&ctx, |ui| {
          Self::build_console(ui)
        });
        CentralPanel::default().show(&ctx, |ui| {
          SidePanel::left("tool_properties").resizable(true).show(&ctx, |ui| {
            self.build_tool_properties(engine.get_asset_cache(), ui)
          });
          CentralPanel::default().show(&ctx, |ui| {
            self.build_dock(&ctx, ui)
          })
        });
      };

      let mut gfx = engine.get_gfx().borrow_mut();
      gfx.begin_frame();
      gfx.end_frame(fun);

      if gfx.should_quit() {
        break 'main_loop;
      }
    }
  }

  fn build_title_menu(&self, ctx: &EguiContext, ui: &mut Ui) {
    ui.horizontal(|ui| {
      ui.image(self.blass.texture_id(ctx), Vec2::new(32.0f32, 32.0f32));
      ui.separator();
      ui.button("File");
      ui.separator();
      ui.button("Project");
      ui.separator();
      ui.button("Preferences");
      ui.separator();
      ui.button("Help");
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

  fn build_console(ui: &mut Ui) {
    ui.label("Console");
    ui.separator();
    ScrollArea::new([false, true]).auto_shrink([false, false]).stick_to_bottom(true)
      .show(ui, |ui| {
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

          ui.separator();
          ui.label(RichText::new(fmt).color(color));
        }
      }
    );
  }

  fn build_dock(&mut self, ctx: &EguiContext, ui: &mut Ui) {
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
        self.dock.dockables[self.dock.focused_dockable].build_content(ui);
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
  selected_asset: Option<String>
}

impl AssetBrowserTool {
  fn new() -> Self {
    Self {
      import_path: String::from(""),
      target_path: String::from(""),
      import_handlers: vec![Box::new(ImageImportHandler::new()), Box::new(MeshImportHandler::new())],
      selected_asset: None
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
      'handler_loop: for handler in &self.import_handlers {
        for (idx, extension) in handler.extensions().iter().enumerate() {
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
        dock.dockables.push(Box::new(AssetEditorDockable::new(asset_cache,
          &(String::from("./ass/") + &String::from(name)))));
      }
    }
  }
}

struct PlayTool;

impl Tool for PlayTool {
  fn name(&self) -> &'static str {
    "Play"
  }

  fn build_tool_properties(&mut self, asset_cache: &RefCell<AssetCache>, dock: &mut Dock, ui: &mut Ui) {
    if ui.button("Play!").clicked() {
      dock.dockables.push(Box::new(PlayDockable { }));
    }
  }
}

struct AssetCacheTool;

impl Tool for AssetCacheTool {
  fn name(&self) -> &'static str {
    "Asset Cache"
  }

  fn build_tool_properties(&mut self, asset_cache: &RefCell<AssetCache>, 
    dock: &mut Dock, ui: &mut Ui) 
  {
    let cache = asset_cache.borrow();
    let assets = cache.borrow_all_assets();
    assets.iter().for_each(|ass| {
      ui.label(ass.0);
    });
  }
}

pub trait Dockable {
  fn title(&self) -> String;
  fn build_content(&self, ui: &mut Ui);
}

struct AssetEditorDockable {
  ass_name: String,
  image: RetainedImage
}

impl AssetEditorDockable {
  fn new(asset_cache: &RefCell<AssetCache>, path: &String) -> Self {
    let mut cache = asset_cache.borrow_mut();
    cache.load_file(path);
    let ass = cache.borrow_asset(path).expect("Couldn't borrow asset!");
    let img = ass.as_any().downcast_ref::<ImageAsset>()
      .expect("Failed to downcast to image!");
    let col_img = 
      ColorImage::from_rgba_premultiplied(
        [img.size.x as usize, img.size.y as usize], 
        &img.data);
    let ret = RetainedImage::from_color_image(path, col_img);
    Self { ass_name: path.clone(), image: ret }
  }
}

impl Dockable for AssetEditorDockable {
  fn title(&self) -> String {
    self.ass_name.clone()
  }

  fn build_content(&self, ui: &mut Ui) {
    SidePanel::new(egui::panel::Side::Right, egui::Id::new("AssetEditorDockable")).show(ui.ctx(), |ui| {
      ui.label(format!("Resolution: {} x {}", self.image.width(), self.image.height()));
    });
    CentralPanel::default().show(ui.ctx(), |ui| {
      ui.centered_and_justified(|ui| {
        let w = self.image.width();
        let h = self.image.height();
        let aspect = w as f32 / h as f32;
        let disp_h = cmp::min(ui.available_height() as usize, h);
        let disp_w = cmp::min(ui.available_width() as usize, 
          (disp_h as f32 * aspect) as usize);
        let disp_h = cmp::min(ui.available_height() as usize, 
          (disp_w as f32 / aspect) as usize);
        ui.image(self.image.texture_id(ui.ctx()), 
          Vec2::new(disp_w as f32, disp_h as f32));
      });
    });
    
  }
}

struct PlayDockable;

impl Dockable for PlayDockable {
  fn title(&self) -> String {
    String::from("Play")
  }

  fn build_content(&self, ui: &mut Ui) {
    ui.label("Play Content");
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
    ser.writer.write_string("Image");
    ser.writer.write_u32(self.img.width());
    ser.writer.write_u32(self.img.height());
    ser.writer.write_bytes(self.img.as_bytes());
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

  fn import(&self, import_path: &String, target_path: &String) {

  }
}
