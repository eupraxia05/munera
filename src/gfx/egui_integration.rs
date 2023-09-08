use egui::epaint::{ImageDelta};
use egui::epaint::textures::TextureFilter;
use egui::{RawInput, Rect, Vec2, Modifiers, Context, TextureId, ImageData, Color32};
use crate::gfx::GfxRuntime;
use crate::math::Vec2i;
use fermium::video::*;
use fermium::*;
extern crate gl;
use gl::types::*;
use std::collections::HashMap;

pub struct EguiIntegration {
  ctx: Context,
  textures: HashMap<TextureId, GLuint>
}

impl EguiIntegration {
  pub fn new() -> Self {
    let frag_shader = compile_shader(gl::VERTEX_SHADER, )

    return Self { ctx: Context::default(), textures: HashMap::new() }
  }

  pub fn run(&mut self, window: *mut SDL_Window) {
    let mut raw_input = RawInput::default();
    let mut w : c_int = 0;
    let mut h : c_int = 0;
    unsafe { SDL_GetWindowSize(window, &mut w, &mut h); }
    raw_input.screen_rect = Some(Rect::from_min_size(Default::default(), 
      Vec2::new(w as f32, h as f32)));
    unsafe { raw_input.focused = 
      (SDL_GetWindowFlags(window) & SDL_WINDOW_INPUT_FOCUS.0 as u32) != 0; }

    let full_output = self.ctx.run(raw_input, |ctx| { test_ui(&self) });

    for delta in full_output.textures_delta.set {
      self.set_texture(delta.0, &delta.1)
    };

    for delta in full_output.textures_delta.free {
      self.free_texture(delta);
    }
  }

  fn set_texture(&mut self, tex_id: TextureId, delta: &ImageDelta) {
    let tex = self.textures.entry(tex_id).or_insert_with(|| unsafe 
    {
      let mut gltex: GLuint = 0;
      gl::CreateTextures(gl::TEXTURE_2D, 1, &mut gltex);
      gltex
    });

    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, *tex);
    }

    let data: Vec<u8>;
    let size: Vec2i;

    match &delta.image {
      ImageData::Color(image) => {
        data = image.pixels.iter()
          .flat_map(|a| a.to_array()).collect();

        size = Vec2i::new(image.size[0] as i32, image.size[1] as i32);
      }

      ImageData::Font(image) => {
        data = image.srgba_pixels(None)
          .flat_map(|a| a.to_array()).collect();
        
        size = Vec2i::new(image.size[0] as i32, image.size[1] as i32);
      }
    }

    unsafe {
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, 
        gl::LINEAR as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, 
        gl::LINEAR as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, 
        gl::CLAMP_TO_EDGE as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, 
        gl::CLAMP_TO_EDGE as i32);

      assert!(gl::GetError() == gl::NO_ERROR);

      gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

      if let Some([x, y]) = delta.pos {
        gl::TexSubImage2D(gl::TEXTURE_2D, 0, x as _, y as _, size.x, size.y, 
          gl::RGBA8, gl::UNSIGNED_BYTE, data.as_ptr().cast::<c_void>());
      } else {
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as i32, size.x, size.y, 0, 
          gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr().cast::<c_void>())
      }

      assert!(gl::GetError() == gl::NO_ERROR);
    }
  }

  fn free_texture(&mut self, tex_id: TextureId) {
    if let Some(old_tex) = self.textures.remove(&tex_id) {
      unsafe {
        gl::DeleteTextures(1, &old_tex);
      }
    }
  }
}

fn test_ui(egui_int: &EguiIntegration) {
  egui::Window::new("Skibadee, skibadanger, I am the rearranger!")
    .show(&egui_int.ctx, |ui| ui.label("Internally coherent!"));
}

fn compile_shader(shader_type: GLenum, code: &str) -> GLuint {
  unsafe {
    let shader = gl::CreateShader(shader_type);

    gl::ShaderSource(shader, 1, 
      &(code.as_bytes().as_ptr().cast()),
      &(code.len().try_into().unwrap()));
    
    gl::CompileShader(shader);

    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

    if success == 0 {
      let mut v: Vec<u8> = Vec::with_capacity(1024);
      let mut log_len = 0_i32;
      gl::GetShaderInfoLog(shader, 1024, &mut log_len, 
        v.as_mut_ptr().cast());
      v.set_len(log_len.try_into().unwrap());
      panic!("Vertex Shader Compile Error: {}", String::from_utf8_lossy(&v));
    }

    shader
  }
}

fn link_program(vert: GLuint, frag: GLuint) -> GLuint {
  unsafe {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vert);
    gl::AttachShader(program, frag);
    gl::LinkProgram(program);
  
    program
  }
}