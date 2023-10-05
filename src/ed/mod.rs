use egui::{TopBottomPanel, Ui, SidePanel, CentralPanel, Context as EguiContext, ScrollArea, TextureHandle, ColorImage, Vec2, Frame, Color32, RichText};

use crate::eng::Engine;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use egui_extras::RetainedImage;
use std::fs;

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
      tools: vec![Box::new(AssetBrowserTool{ }), Box::new(PlayTool{ })],
      selected_tool_idx: None,
    dock: Dock::new() }
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
            self.build_tool_properties(ui)
          });
          CentralPanel::default().show(&ctx, |ui| {
            self.build_dock(&ctx, ui)
          })
        });
      };

      engine.get_gfx().begin_frame();
      engine.get_gfx().end_frame(fun);

      if engine.get_gfx().should_quit() {
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

  fn build_tool_properties(&mut self, ui: &mut Ui) {
    ScrollArea::new([false, true]).show(ui, |ui| {
      if self.selected_tool_idx.is_some() {
        let tool = &self.tools[self.selected_tool_idx.unwrap()];
        ui.label(tool.name());
        ui.separator();
        tool.build_tool_properties(&mut self.dock, ui);
      } else {
        ui.label("Tool Properties");
        ui.separator();
      }
    });
  }

  fn build_console(ui: &mut Ui) {
    ui.label("Console");
    ui.separator();
    ScrollArea::new([false, true]).auto_shrink([false, false]).show(ui, |ui| {
      ui.label(RichText::new("I am the very image of a modern major-general!"));
    });
  }

  fn build_dock(&mut self, ctx: &EguiContext, ui: &mut Ui) {
    TopBottomPanel::top("dock_tabs").show(ctx, |ui| {
      ui.horizontal(|ui| {
        for (idx, dockable) in self.dock.dockables.iter().enumerate() {
          if ui.button(dockable.title()).clicked() {
            self.dock.focused_dockable = idx;
          }
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
  fn build_tool_properties(&self, dock: &mut Dock, ui: &mut Ui);
}

struct AssetBrowserTool;

impl Tool for AssetBrowserTool {
  fn name(&self) -> &'static str {
    "Asset Browser"
  }

  fn build_tool_properties(&self, dock: &mut Dock, ui: &mut Ui) {
    let paths = fs::read_dir("./ass/").unwrap();

    for path in paths {
      ui.label(path.unwrap().file_name().to_str().unwrap());
    }

    if ui.button("Beep!").clicked() {
      dock.dockables.push(Box::new(AssetEditorDockable { }))
    }
  }
}

struct PlayTool;

impl Tool for PlayTool {
  fn name(&self) -> &'static str {
    "Play"
  }

  fn build_tool_properties(&self, dock: &mut Dock, ui: &mut Ui) {
    if ui.button("Play!").clicked() {
      dock.dockables.push(Box::new(PlayDockable { }));
    }
  }
}

pub trait Dockable {
  fn title(&self) -> String;
  fn build_content(&self, ui: &mut Ui);
}

struct AssetEditorDockable;

impl Dockable for AssetEditorDockable {
  fn title(&self) -> String {
    String::from("Asset Editor")
  }

  fn build_content(&self, ui: &mut Ui) {
    ui.label("Asset Editor Content");
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