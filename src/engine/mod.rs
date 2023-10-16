use std::any::TypeId;
use hecs::{EntityRef, Entity, World};
use serde::{Serialize, Deserialize, ser::SerializeMap};
use mac::{Comp, define_comps};
use std::cell::RefCell;

use crate::assets;

pub mod logger;

// A base trait to generate metadata for a component type.
pub trait Comp {
  fn ent_has(ent: EntityRef) -> bool;
  fn ent_add(world: &mut World, ent: Entity);
  fn ent_rem(world: &mut World, ent: Entity);
}

/// Utility component to tag entities with a human-friendly name.
#[derive(Comp, Serialize, Deserialize, Default)]
pub struct NameComp {
  pub name: String
}

/// Contains metadata defined for a particular component type.
pub struct CompType {
  pub name: String,
  pub type_id: TypeId,
  pub ent_has: fn(EntityRef) -> bool,
  pub ent_add: fn(&mut World, Entity),
  pub ent_rem: fn(&mut World, Entity),
}

impl CompType {
  fn new<T>(name: &str) -> Self
    where T: Comp + 'static {
    Self { name: name.to_string(), type_id: TypeId::of::<T>(), ent_has: T::ent_has, 
      ent_add: T::ent_add, ent_rem: T::ent_rem }
  }
}

/// A context containing all engine systems, assets, and metadata.
pub struct Engine {
  comp_types: Vec<CompType>,
  asset_cache: RefCell<assets::AssetCache>,
  event_loop: winit::event_loop::EventLoop<()>,
  window: winit::window::Window,
  instance: wgpu::Instance,
  device: wgpu::Device,
  queue: wgpu::Queue,
  surface: wgpu::Surface,
  surface_config: wgpu::SurfaceConfiguration,
  egui_winit_plat: egui_winit_platform::Platform,
  egui_rpass: egui_wgpu_backend::RenderPass
}

impl Engine {
  pub fn new() -> Self {
    log::set_logger(&logger::LOGGER)
      .map(|()| log::set_max_level(log::LevelFilter::Info))
      .expect("Couldn't set logger!");

    log::info!("Initializing engine...");

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().with_title("Munera").with_maximized(true).build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface(&window).unwrap() };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      compatible_surface: Some(&surface),
      force_fallback_adapter: false
    })).unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::default(),
        limits: wgpu::Limits::default(),
        label: None,
      }, None
    )).unwrap();

    let size = window.inner_size();
    let surface_format = surface.get_capabilities(&adapter).formats[0];
    let surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width as u32,
      height: size.height as u32,
      present_mode: wgpu::PresentMode::AutoVsync,
      alpha_mode: wgpu::CompositeAlphaMode::Opaque,
      view_formats: vec![]
    };
    surface.configure(&device, &surface_config);

    let egui_winit_plat = egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
      physical_width: size.width as u32,
      physical_height: size.height as u32,
      scale_factor: window.scale_factor(),
      font_definitions: Default::default(),
      style: Default::default()
    });

    let egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);

    Self { 
      comp_types: define_comps!(), 
      asset_cache: RefCell::new(assets::AssetCache::new()),
      event_loop,
      window,
      instance,
      device,
      queue,
      surface,
      surface_config,
      egui_winit_plat,
      egui_rpass
    }
  }

  pub fn run<AppType: App + 'static>(mut self, mut app: AppType) {
    let start_time = std::time::Instant::now();
    self.event_loop.run(move |event, _, control_flow| {
      control_flow.set_poll();

      self.egui_winit_plat.handle_event(&event);

      match event {
        winit::event::Event::WindowEvent { event, .. } => {
          match event {
            winit::event::WindowEvent::CloseRequested => {
              control_flow.set_exit();
            },
            winit::event::WindowEvent::Resized(size) => {
              if size.width > 0 && size.height > 0 {
                self.surface_config.width = size.width;
                self.surface_config.height = size.height;
                self.surface.configure(&self.device, &self.surface_config);
              }
            }
            _ => ()
          }
        },
        winit::event::Event::MainEventsCleared => {
          self.egui_winit_plat.update_time(start_time.elapsed().as_secs_f64());

          let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
              return;
            },
            Err(e) => {
              log::error!("Dropped frame with error: {}", e);
              return;
            }
          };

          app.tick(0.0, &self.device, &mut self.egui_rpass, &self.queue);

          let output_view = output_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

          self.egui_winit_plat.begin_frame();

          app.build_ui(&self.asset_cache, &self.egui_winit_plat.context(), &self.device);

          let full_output = self.egui_winit_plat.end_frame(Some(&self.window));
          let paint_jobs = self.egui_winit_plat.context().tessellate(full_output.shapes);

          let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder")
          });

          let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: self.window.inner_size().width,
            physical_height: self.window.inner_size().height,
            scale_factor: self.window.scale_factor() as f32
          };

          self.egui_rpass.add_textures(&self.device, &self.queue, &full_output.textures_delta)
            .expect("Failed to add textures!");

          self.egui_rpass.update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

          self.egui_rpass.execute(&mut encoder, &output_view, &paint_jobs, &screen_descriptor, Some(wgpu::Color::BLACK))
            .expect("Failed to execute egui rendering!");

          self.queue.submit(std::iter::once(encoder.finish()));

          output_frame.present();

          self.egui_rpass.remove_textures(full_output.textures_delta)
            .expect("Failed to remove textures!");
        }
        _ => ()
      }
    });
  }

  pub fn get_comp_types(&self) -> &Vec<CompType> {
    &self.comp_types
  }

  pub fn get_asset_cache(&self) -> &RefCell<assets::AssetCache> {
    &self.asset_cache
  }
}

pub trait App {
  fn tick(&mut self, dt: f32, device: &wgpu::Device, egui_rpass: &mut egui_wgpu_backend::RenderPass, queue: &wgpu::Queue);
  fn build_ui(&mut self, asset_cache: &RefCell<assets::AssetCache>, egui_context: &egui::Context, device: &wgpu::Device);
}
