pub struct IsoRenderer {
  pipeline: wgpu::RenderPipeline
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
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        buffers: &[],
        entry_point: "vs_main"
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(output_format)]
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
    });

    Self { pipeline, }
  }

  pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, output_tex_view: &wgpu::TextureView) 
  {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("IsoRenderer Render Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: output_tex_view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          store: true
        }
      })],
      depth_stencil_attachment: None,
    });

    render_pass.set_pipeline(&self.pipeline);
    render_pass.draw(0..3, 0..1);
  }
}