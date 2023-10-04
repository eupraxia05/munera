use egui::{TopBottomPanel, Ui, SidePanel, CentralPanel, Context as EguiContext, ScrollArea, TextureHandle, ColorImage, Vec2, Frame, Color32, RichText};

use crate::eng::Engine;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use egui_extras::RetainedImage;

const TOOLBAR_WIDTH: f32 = 64.0f32;
const MIN_CONSOLE_HEIGHT: f32 = 256.0f32;

pub struct Editor<'a> {
  engine: RefCell<&'a mut Engine>,
  blass: RetainedImage,
  tools: Vec<Box<dyn Tool>>
}

impl<'a> Editor<'a> {
  pub fn new(engine: &'a mut Engine) -> Self {
    Self { engine: RefCell::new(engine), 
      blass: RetainedImage::from_image_bytes("tuwuck.png", include_bytes!("../../ass/tuwuck.png"))
        .expect("Failed to load image!"),
      tools: vec![Box::new(AssetBrowserTool{ }), Box::new(PlayTool{ })] }
  }

  pub fn run(&'a mut self) {
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
          SidePanel::left("inspector").resizable(true).show(&ctx, |ui| {
            Self::build_inspector(ui)
          });
          CentralPanel::default().show(&ctx, |ui| {
            Self::build_dock(&ctx, ui)
          })
        });
      };

      let mut eng = self.engine.borrow_mut();
      eng.get_gfx().begin_frame();
      eng.get_gfx().end_frame(fun);

      if eng.get_gfx().should_quit() {
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

  fn build_toolbar(&self, ui: &mut Ui) {
    ui.label("Toolbar");
    ui.separator();
    for tool in &self.tools {
      ui.button(tool.name());
    }
  }

  fn build_inspector(ui: &mut Ui) {
    ScrollArea::new([false, true]).show(ui, |ui| {
      ui.label("Inspector");
      ui.separator();
    });
  }

  fn build_console(ui: &mut Ui) {
    ui.label("Console");
    ui.separator();
    ScrollArea::new([false, true]).auto_shrink([false, false]).show(ui, |ui| {
      ui.label(RichText::new("I am the very image of a modern major-general!"));
    });
  }

  fn build_dock(ctx: &EguiContext, ui: &mut Ui) {
    TopBottomPanel::top("dock_tabs").show(ctx, |ui| {
      ui.label("Tabs");
    });
    CentralPanel::default().show(ctx, |ui| {
      ui.label("Dockable Content");
    });
  }
}

trait Tool {
  fn name(&self) -> &'static str;
  fn image_file(&self) -> &'static str;
}

struct AssetBrowserTool;

impl Tool for AssetBrowserTool {
  fn name(&self) -> &'static str {
    "Asset Browser"
  }

  fn image_file(&self) -> &'static str {
    "asset_browser.png"
  }
}

struct PlayTool;

impl Tool for PlayTool {
  fn name(&self) -> &'static str {
    "Play"
  }

  fn image_file(&self) -> &'static str {
    "play.png"
  }
}