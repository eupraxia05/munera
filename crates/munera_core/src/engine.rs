
/// A base trait covering object-safe parts of the component interface.
/// Additional non-object-safe functionality is exposed via [`CompExt`].
/// 
/// To create a component type from a struct, use `#[derive(munera_macros::Comp)]`. 
/// The generated [`CompType`]s can be accessed with 
/// `inventory::iter::<crate::engine::CompType>`.
#[typetag::serde(tag = "type")]
pub trait Comp: erased_serde::Serialize + std::marker::Sync + std::marker::Send 
  + 'static 
{
  fn as_any(&self) -> &dyn std::any::Any;
}

/// Exposes non-object-safe parts of the component interface. See `[Comp]`.
pub trait CompExt : Comp + Default + Clone + for<'de> serde::Deserialize<'de>
  + CompInspect
{

}

pub trait CompInspect {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool;
}

/// Contains metadata defined for a particular component type. Privately, this
/// implements a bunch of generic functions and stores them as function
/// pointers. The `Comp` macro stores this for a component type, and they are
/// iterable using `inventory::iter::<crate::engine::CompType>`.
#[derive(PartialEq, Clone)]
pub struct CompType {
  /// The name of the component type.
  pub name: String,

  /// The type id of the component type.
  pub type_id: std::any::TypeId,

  /// A function to check if an entity has this component type.
  pub ent_has: fn(hecs::EntityRef) -> bool,

  /// A function to add this component (constructed with `Default::default()`)
  /// to a given entity.
  pub ent_add: fn(&mut hecs::World, hecs::Entity),

  /// A function to remove this component from a given entity.
  pub ent_rem: fn(&mut hecs::World, hecs::Entity),

  /// A function to get this component generically, copied, from the component.
  pub ent_get: fn(&hecs::World, hecs::Entity) -> Box<dyn Comp>,

  /// A function to copy to the component from a deserialized component.
  // !TODO: probably a better solution here
  pub ent_deserialize: fn(&mut hecs::EntityBuilder, &Box<dyn Comp>),

  /// A function to show the component in the inspector.
  pub ent_inspect: fn(&mut hecs::World, hecs::Entity, &mut egui::Ui) -> bool
}

impl CompType {
  pub fn new<T>(name: &str) -> Self
    where T: CompExt  {
    Self { name: name.to_string(), type_id: std::any::TypeId::of::<T>(), 
      ent_has: Self::ent_has::<T>, ent_add: Self::ent_add::<T>, 
      ent_rem: Self::ent_rem::<T>, ent_get: Self::ent_get::<T>,
      ent_deserialize: Self::ent_deserialize::<T>, 
      ent_inspect: Self::ent_inspect::<T> }
  }

  /// Finds the CompType corresponding to a particular static type.
  pub fn find<T>() -> Option<Self>
    where T: CompExt
  {
    for comp_type in inventory::iter::<CompType>() {
      if comp_type.type_id == std::any::TypeId::of::<T>() {
        return Some(comp_type.clone());
      }
    }
    None
  }

  fn ent_has<T>(ent: hecs::EntityRef) -> bool
    where T: CompExt
  {
    ent.has::<T>()
  }

  fn ent_add<T>(world: &mut hecs::World, entity: hecs::Entity) 
    where T: CompExt
  {
    if let Err(e) = world.insert_one(entity, T::default()) {
      log::error!("Couldn't add component to entity: {}", e);
    }
  }

  fn ent_rem<T>(world: &mut hecs::World, entity: hecs::Entity)
    where T: CompExt
  {
    if let Err(e) = world.remove_one::<T>(entity) {
      log::error!("Couldn't remove component from entity: {}", e);
    }
  }

  fn ent_get<T>(world: &hecs::World, entity: hecs::Entity) -> Box<dyn Comp>
    where T: CompExt
  {
    Box::new((*world.get::<&T>(entity).unwrap()).clone())
  }

  fn ent_deserialize<T>(entity: &mut hecs::EntityBuilder, value: &Box<dyn Comp>) 
    where T: CompExt
  {
    entity.add::<T>(value.as_ref().as_any().downcast_ref::<T>()
      .unwrap().clone());
  }

  fn ent_inspect<T>(world: &mut hecs::World, entity: hecs::Entity, 
    ui: &mut egui::Ui) -> bool
    where T: CompExt
  {
    if let Ok(mut comp) = world.get::<&mut T>(entity) {
      comp.inspect(ui);
      false
    } else {
      false
    }
  }
}

inventory::collect!(CompType);

/// A context containing all engine systems, assets, and metadata.
pub fn run<'a, AppType: App<'a> + 'static>() {
  let asset_cache = std::cell::RefCell::new(munera_assets::AssetCache::new());
  
  log::set_logger(&crate::logger::LOGGER)
    .map(|()| log::set_max_level(log::LevelFilter::Info))
    .expect("Couldn't set logger!");

  log::info!("Initializing engine...");

  let event_loop = winit::event_loop::EventLoop::new();
  let window = winit::window::WindowBuilder::new().with_title("Munera")
    .build(&event_loop).unwrap();

  let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::VULKAN,
    dx12_shader_compiler: wgpu::Dx12Compiler::Fxc
  });
  let surface = unsafe { instance.create_surface(&window).unwrap() };

  let adapter = pollster::block_on(instance.request_adapter(
    &wgpu::RequestAdapterOptions {
    power_preference: wgpu::PowerPreference::HighPerformance,
    compatible_surface: Some(&surface),
    force_fallback_adapter: false
  })).unwrap();

  let mut limits = wgpu::Limits::default();
  limits.max_push_constant_size = 128;

  let (device, queue) = pollster::block_on(adapter.request_device(
    &wgpu::DeviceDescriptor {
      features: wgpu::Features::default() | wgpu::Features::PUSH_CONSTANTS 
        | wgpu::Features::POLYGON_MODE_POINT,
      limits,
      label: None,
    }, None
  )).unwrap();

  device.on_uncaptured_error(Box::new(|error| {
    log::error!("{}", error);
  }));

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

  let mut egui_winit_plat = egui_winit_platform::Platform::new(
    egui_winit_platform::PlatformDescriptor {
    physical_width: size.width as u32,
    physical_height: size.height as u32,
    scale_factor: window.scale_factor(),
    font_definitions: Default::default(),
    style: Default::default()
  });

  let mut egui_rpass = egui_wgpu_backend::RenderPass::new(&device, 
    surface_format, 1);
  
  let mut app = AppType::default();
  app.init(&window);

  let start_time = std::time::Instant::now();
  event_loop.run(move |event, _, control_flow| {

    control_flow.set_poll();

    egui_winit_plat.handle_event(&event);

    match event {
      winit::event::Event::WindowEvent { event, .. } => {
        match event {
          winit::event::WindowEvent::CloseRequested => {
            app.exit(&window);
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

        let output_view = output_frame.texture.create_view(
          &wgpu::TextureViewDescriptor::default());

        app.tick(&mut AppTickInfo {
          dt: 0.0, 
          device: &device, 
          asset_cache: &asset_cache, 
          egui_rpass: &mut egui_rpass, 
          queue: &queue, 
          output_tex_view: &output_view
        });

        egui_winit_plat.begin_frame();

        app.build_ui(&AppBuildUiInfo {
          asset_cache: &asset_cache, 
          egui_context: &egui_winit_plat.context(), 
          device: &device 
        });

        let full_output = egui_winit_plat.end_frame(Some(&window));
        let paint_jobs = egui_winit_plat.context()
          .tessellate(full_output.shapes);

        let mut encoder = device.create_command_encoder(
          &wgpu::CommandEncoderDescriptor {
          label: Some("encoder")
        });

        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
          physical_width: window.inner_size().width,
          physical_height: window.inner_size().height,
          scale_factor: window.scale_factor() as f32
        };

        egui_rpass.add_textures(&device, &queue, &full_output.textures_delta)
          .expect("Failed to add textures!");

        egui_rpass.update_buffers(&device, &queue, &paint_jobs, 
          &screen_descriptor);

        egui_rpass.execute(&mut encoder, &output_view, &paint_jobs, 
          &screen_descriptor, Some(wgpu::Color::BLACK))
          .expect("Failed to execute egui rendering!");

        queue.submit(std::iter::once(encoder.finish()));

        output_frame.present();

        egui_rpass.remove_textures(full_output.textures_delta)
          .expect("Failed to remove textures!");

        if app.should_quit() {
          app.exit(&window);
          control_flow.set_exit();
        }
      }
      _ => ()
    }
  });
}

pub struct AppTickInfo<'a> {
  pub dt: f32,
  pub device: &'a wgpu::Device,
  pub asset_cache: &'a std::cell::RefCell<munera_assets::AssetCache>,
  pub egui_rpass: &'a mut egui_wgpu_backend::RenderPass,
  pub queue: &'a wgpu::Queue,
  pub output_tex_view: &'a wgpu::TextureView,
}

pub struct AppBuildUiInfo<'a> {
  pub asset_cache: &'a std::cell::RefCell<munera_assets::AssetCache>,
  pub egui_context: &'a egui::Context,
  pub device: &'a wgpu::Device,
}

pub trait App<'a>: Default {
  fn tick(&mut self, _tick_info: &mut AppTickInfo) { }
  fn build_ui(&mut self, _build_ui_info: &AppBuildUiInfo) { }
  fn init(&mut self, _window: &winit::window::Window) { }
  fn exit(&mut self, _window: &winit::window::Window) { }
  fn should_quit(&mut self) -> bool { false }
}
