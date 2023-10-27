use wgpu::util::DeviceExt;

use crate::engine::CompInspect;


pub struct IsoRenderer {
  pipeline: wgpu::RenderPipeline,
  buffer: wgpu::Buffer
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
        targets: &[Some(output_format)]
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

    Self { pipeline, buffer }
  }

  pub fn render(&mut self, world: &hecs::World, encoder: &mut wgpu::CommandEncoder, output_tex_view: &wgpu::TextureView,
    screen_size: crate::math::Vec2u) 
  {
    if let Some(scene_ent) = world.iter().find(|ent| ent.has::<SceneComp>()) {
      let scene_comp = scene_ent.get::<&SceneComp>().unwrap();
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("IsoRenderer Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: output_tex_view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(scene_comp.background_color.into()),
            store: true
          }
        })],
        depth_stencil_attachment: None,
      });
  
      let push_constants = PushConstants {
        screen_size
      };
  
      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_vertex_buffer(0, self.buffer.slice(0..));
      render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::bytes_of(&push_constants));
      render_pass.draw(0..6, 0..25);
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
  background_color: crate::math::Color
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
    modified
  }
}