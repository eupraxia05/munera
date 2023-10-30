use wgpu::util::DeviceExt;

use crate::engine::CompInspect;


pub struct IsoRenderer {
  pipeline: wgpu::RenderPipeline,
  buffer: wgpu::Buffer,
  gbuffer: Gbuffer
}

impl IsoRenderer {
  pub fn new(device: &wgpu::Device, output_format: wgpu::ColorTargetState) -> Self {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("triangle shader"),
      source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../ass/shader.wgsl")))
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
      label: None,
      bind_group_layouts: &[],
      push_constant_ranges: &[wgpu::PushConstantRange {
        stages: wgpu::ShaderStages::VERTEX,
        range: 0..std::mem::size_of::<PushConstants>() as u32
      }],
    });

    let mut tiles = Vec::new();
    tiles.resize(25, InstanceData { 
      position: crate::math::Vec3f::new(0.0, 0.0, 0.0),
      _padding: 0.0,
      color: crate::math::Color::new(0.0, 0.0, 0.0, 0.0)
    });

    for i in 0..5 {
      for j in 0..5 {
        let tile = tiles.get_mut(i + j * 5).unwrap();
        tile.position = crate::math::Vec3f::new(i as f32 - 2.0f32, j as f32 - 2.0f32, 0.0);
        tile.color = crate::math::Color::new(i as f32 / 4.0, j as f32 / 4.0, 0.0, 1.0);
      }
    }

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: bytemuck::cast_slice(&tiles),
      usage: wgpu::BufferUsages::VERTEX
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        buffers: &[wgpu::VertexBufferLayout{
          array_stride: std::mem::size_of::<InstanceData>() as u64,
          step_mode: wgpu::VertexStepMode::Instance,
          attributes: &[wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x3,
              offset: 0,
              shader_location: 0
            },
            wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x4,
              offset: memoffset::offset_of!(InstanceData, color) as u64,
              shader_location: 1
            }
          ]
        }],
        entry_point: "vs_main"
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(output_format.clone())]
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Cw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
    });

    let gbuffer = Gbuffer::new(device, output_format);

    Self { pipeline, buffer, gbuffer }
  }

  pub fn render(&mut self, world: &hecs::World, device: &wgpu::Device, 
    encoder: &mut wgpu::CommandEncoder, output_tex_view: &wgpu::TextureView,
    screen_size: crate::math::Vec2u) 
  {
    if let Some(scene_ent) = world.iter().find(|ent| ent.has::<SceneComp>()) {
      let scene_comp = scene_ent.get::<&SceneComp>().unwrap();

      {
        self.gbuffer.update_res(device, screen_size, scene_comp.pixel_scale);
        let mut render_pass = self.gbuffer.begin_scene_render_pass(encoder, scene_comp.background_color);
  
        let push_constants = PushConstants {
          screen_size: self.gbuffer.current_res()
        };
    
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.buffer.slice(0..));
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::bytes_of(&push_constants));
        render_pass.draw(0..6, 0..25);
      }
      
      self.gbuffer.perform_upscale(encoder, output_tex_view)
    }
  }
}

#[derive(bytemuck::NoUninit, Clone, Copy)]
#[repr(C)]
struct PushConstants {
  screen_size: crate::math::Vec2u
}

#[derive(bytemuck::NoUninit, Clone, Copy)]
#[repr(C)]
struct InstanceData {
  position: crate::math::Vec3f,
  _padding: f32,
  color: crate::math::Color
}

#[derive(Default, serde::Serialize, serde::Deserialize, mac::Comp, RTTI, Clone)]
struct SceneComp {
  background_color: crate::math::Color,

  #[serde(default)]
  pixel_scale: u32
}

impl CompInspect for SceneComp {
  fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
    let mut modified = false;
    ui.horizontal(|ui| {
      ui.label("Background Color");
      let mut col = &mut [self.background_color.r, self.background_color.g, self.background_color.b, 
        self.background_color.a];
      if ui.color_edit_button_rgba_unmultiplied(&mut col).changed() {
        self.background_color.r = col[0];
        self.background_color.g = col[1];
        self.background_color.b = col[2];
        self.background_color.a = col[3];
        modified = true;
      }
    });
    ui.horizontal(|ui| {
      ui.label("Pixel Scale");
      if ui.add(egui::DragValue::new(&mut self.pixel_scale)).changed() {
        modified = true;
      }
    });
    modified
  }
}

struct Gbuffer {
  texture: Option<wgpu::Texture>,
  texture_view: Option<wgpu::TextureView>,
  bind_group_layout: wgpu::BindGroupLayout,
  bind_group: Option<wgpu::BindGroup>,
  upscale_pipeline_layout: wgpu::PipelineLayout,
  target_res: crate::math::Vec2u,
  upscale_pipeline: wgpu::RenderPipeline,
  upscale_sampler: wgpu::Sampler,
}

impl Gbuffer {
  fn new(device: &wgpu::Device, output_format: wgpu::ColorTargetState) -> Self {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: None,
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture { 
            sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
            view_dimension: wgpu::TextureViewDimension::D2, 
            multisampled: (false) 
          },
          count: None
        }, wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None
        }
      ],
    });

    let upscale_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[]
    });

    let upscale_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../ass/upscale_shader.wgsl"))),
    });

    let upscale_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&upscale_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &upscale_shader_module,
        entry_point: "vs_main",
        buffers: &[]
      },
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Cw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      fragment: Some(wgpu::FragmentState {
        module: &upscale_shader_module,
        entry_point: "fs_main",
        targets: &[Some(output_format)]
      }),
      multiview: None
    });

    let upscale_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: None,
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Nearest,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      lod_min_clamp: 0.0f32,
      lod_max_clamp: 0.0f32,
      compare: None,
      anisotropy_clamp: 1,
      border_color: None
    });

    Gbuffer {
      texture: None,
      texture_view: None,
      bind_group_layout,
      bind_group: None,
      upscale_pipeline_layout,
      target_res: Default::default(),
      upscale_pipeline,
      upscale_sampler
    }
  }

  fn update_res(&mut self, device: &wgpu::Device, target_res: crate::math::Vec2u, mut pixel_scale: u32) {
    assert!(self.texture.is_some() == self.texture_view.is_some() 
      && self.texture.is_some() == self.bind_group.is_some());
    
    if pixel_scale == 0 {
      pixel_scale = 1;
    }

    let scaled_res = target_res / pixel_scale;
    let out_of_date = if self.texture.is_none() {
      true 
    } else {
      let tex = self.texture.as_ref().unwrap();
      let tex_size = crate::math::Vec2u::new(tex.width(), tex.height());
      tex_size != scaled_res
    };

    if !out_of_date {
      return
    }

    if self.texture.is_some() {
      self.texture.as_ref().unwrap().destroy();
      self.texture = None;
      self.texture_view = None;
      self.bind_group = None;
    }

    self.texture = Some(device.create_texture(&wgpu::TextureDescriptor {
      label: None,
      size: wgpu::Extent3d {
        width: scaled_res.x,
        height: scaled_res.y,
        depth_or_array_layers: 1
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba16Float,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[]
    }));

    self.texture_view = Some(self.texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor {
      label: None,
      format: Some(wgpu::TextureFormat::Rgba16Float),
      dimension: Some(wgpu::TextureViewDimension::D2),
      aspect: wgpu::TextureAspect::All,
      base_mip_level: 0,
      mip_level_count: None,
      base_array_layer: 0,
      array_layer_count: None
    }));

    self.target_res = target_res;

    self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &self.bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(self.texture_view.as_ref().unwrap())
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&self.upscale_sampler)
        }
      ],
    }));
  }

  fn begin_scene_render_pass<'a, 'b>(&'b self, encoder: &'a mut wgpu::CommandEncoder, clear_color: crate::math::Color)
    -> wgpu::RenderPass<'a>
    where 'b: 'a
  {
    assert!(self.texture.is_some() && self.texture_view.is_some());

    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: None,
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &self.texture_view.as_ref().unwrap(),
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(clear_color.into()),
          store: true
        }
      })],
      depth_stencil_attachment: None
    })
  }

  fn perform_upscale(&self, encoder: &mut wgpu::CommandEncoder, output_view: &wgpu::TextureView) {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: None,
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: output_view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: true
        }
      })],
      depth_stencil_attachment: None
    });

    rpass.set_pipeline(&self.upscale_pipeline);
    rpass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
    rpass.draw(0..6, 0..1);
  }

  fn current_res(&self) -> crate::math::Vec2u {
    if let Some(tex) = self.texture.as_ref() {
      crate::math::Vec2u::new(tex.width(), tex.height())
    } else {
      crate::math::Vec2u::default()
    }
  }
}