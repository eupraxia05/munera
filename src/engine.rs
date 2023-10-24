use std::any::TypeId;
use hecs::{EntityRef, Entity, World};
use serde::{Serialize, Deserialize, ser::SerializeMap};
use mac::{Comp, define_comps};
use std::cell::RefCell;

use crate::assets;

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
pub struct Engine;

impl Engine {
  pub fn run<'a, AppType: App<'a> + 'static>() {
    let comp_types = define_comps!(); 
    let asset_cache = RefCell::new(assets::AssetCache::new());
    
    log::set_logger(&crate::logger::LOGGER)
      .map(|()| log::set_max_level(log::LevelFilter::Info))
      .expect("Couldn't set logger!");

    log::info!("Initializing engine...");

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().with_title("Munera").with_maximized(true).build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::VULKAN,
      dx12_shader_compiler: wgpu::Dx12Compiler::Fxc
    });
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
    let mut surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width as u32,
      height: size.height as u32,
      present_mode: wgpu::PresentMode::AutoVsync,
      alpha_mode: wgpu::CompositeAlphaMode::Opaque,
      view_formats: vec![]
    };
    surface.configure(&device, &surface_config);

    let mut egui_winit_plat = egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
      physical_width: size.width as u32,
      physical_height: size.height as u32,
      scale_factor: window.scale_factor(),
      font_definitions: Default::default(),
      style: Default::default()
    });

    let mut egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);
    
    let mut app = AppType::default();

    let start_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {

      control_flow.set_poll();

      egui_winit_plat.handle_event(&event);

      match event {
        winit::event::Event::WindowEvent { event, .. } => {
          match event {
            winit::event::WindowEvent::CloseRequested => {
              control_flow.set_exit();
            },
            winit::event::WindowEvent::Resized(size) => {
              if size.width > 0 && size.height > 0 {
                surface_config.width = size.width;
                surface_config.height = size.height;
                surface.configure(&device, &surface_config);
              }
            }
            _ => ()
          }
        },
        winit::event::Event::MainEventsCleared => {
          egui_winit_plat.update_time(start_time.elapsed().as_secs_f64());

          let output_frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
              return;
            },
            Err(e) => {
              log::error!("Dropped frame with error: {}", e);
              return;
            }
          };

          let output_view = output_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

          app.tick(0.0, &device, &asset_cache, &mut egui_rpass, &queue, &output_view);

          egui_winit_plat.begin_frame();

          app.build_ui(&asset_cache, &egui_winit_plat.context(), &device);

          let full_output = egui_winit_plat.end_frame(Some(&window));
          let paint_jobs = egui_winit_plat.context().tessellate(full_output.shapes);

          let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder")
          });

          let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: window.inner_size().width,
            physical_height: window.inner_size().height,
            scale_factor: window.scale_factor() as f32
          };

          egui_rpass.add_textures(&device, &queue, &full_output.textures_delta)
            .expect("Failed to add textures!");

          egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

          egui_rpass.execute(&mut encoder, &output_view, &paint_jobs, &screen_descriptor, Some(wgpu::Color::BLACK))
            .expect("Failed to execute egui rendering!");

          queue.submit(std::iter::once(encoder.finish()));

          output_frame.present();

          egui_rpass.remove_textures(full_output.textures_delta)
            .expect("Failed to remove textures!");
        }
        _ => ()
      }
    });
  }
}

pub struct AppRunner<'a, AppType>
  where AppType: App<'a> + 'static
{
  phantom_data: std::marker::PhantomData<&'a AppType>
}

impl<'a, AppType> AppRunner<'a, AppType> 
  where AppType: App<'a>
{
  pub fn new() -> Self {
    Self { 
      phantom_data: Default::default()
    }
  }

  pub fn run<'b>(&'b mut self) {
    let mut app = AppType::default();
    Engine::run::<AppType>();
  }
}

pub trait App<'a>: Default {
  fn tick(&mut self, dt: f32, device: &wgpu::Device, asset_cache: &RefCell<assets::AssetCache>, 
    egui_rpass: &mut egui_wgpu_backend::RenderPass, queue: &wgpu::Queue, output_tex_view: &wgpu::TextureView);
  fn build_ui(&mut self, asset_cache: &RefCell<assets::AssetCache>, egui_context: &egui::Context, device: &wgpu::Device);
}
