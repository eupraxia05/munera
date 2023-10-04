use std::ffi::{CString, CStr};
use std::result::Result;
use egui::Context;
use fermium::prelude::*;
use fermium::*;
use fermium::video::*;
extern crate gl;
use gl::types::*;
use std::mem::{size_of, size_of_val};
use crate::math::Vec2i;

mod egui_integration;

use self::egui_integration::EguiIntegration;

pub struct GfxRuntime {
  window: *mut SDL_Window,
  gl_context: SDL_GLContext,
  vao: GLuint,
  vbo: GLuint,
  egui: EguiIntegration,
  shader_program: GLuint,
  should_quit: bool,
  events: Vec<SDL_Event>
}

impl GfxRuntime {
  pub fn new() -> Self {
    unsafe { 
      SDL_Init(SDL_INIT_EVERYTHING);
      SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
      SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 6);

      let window = SDL_CreateWindow(CString::new("Munera").unwrap().as_ptr(), 
        SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, 1920, 1080, 
        SDL_WINDOW_OPENGL.0);

      let gl_context = SDL_GL_CreateContext(window);

      gl::load_with(|s| SDL_GL_GetProcAddress(
        CString::new(s).unwrap().as_ptr()));

      gl::Enable(gl::DEBUG_OUTPUT);
      gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
      gl::DebugMessageCallback(Some(Self::debug_message_callback), std::ptr::null());

      let mut vao: GLuint = 0;
      gl::GenVertexArrays(1, &mut vao);

      let mut vbo: GLuint = 0;
      gl::GenBuffers(1, &mut vbo);

      gl::ClearColor(0.05, 0.05, 0.05, 1.0);
      gl::BindVertexArray(vao);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

      type Vertex = [f32; 3];
      const VERTICES: [Vertex; 3] =
        [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

      gl::BufferData(gl::ARRAY_BUFFER, size_of_val(&VERTICES) as isize,
        VERTICES.as_ptr().cast(), gl::STATIC_DRAW);

      gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 
        size_of::<Vertex>().try_into().unwrap(), 0 as *const _);
      
      gl::EnableVertexAttribArray(0);

      let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
      const VERT_SHADER: &str = r#"#version 460 core
        layout (location = 0) in vec3 pos;
        void main() {
            gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
        }
        "#;

      gl::ShaderSource(vertex_shader, 1, 
        &(VERT_SHADER.as_bytes().as_ptr().cast()),
        &(VERT_SHADER.len().try_into().unwrap()));
      
      gl::CompileShader(vertex_shader);

      let mut success = 0;
      gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);

      if success == 0 {
        let mut v: Vec<u8> = Vec::with_capacity(1024);
        let mut log_len = 0_i32;
        gl::GetShaderInfoLog(vertex_shader, 1024, &mut log_len, 
          v.as_mut_ptr().cast());
        v.set_len(log_len.try_into().unwrap());
        panic!("Vertex Shader Compile Error: {}", String::from_utf8_lossy(&v));
      }

      let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
      const FRAG_SHADER: &str = r#"#version 460 core
        out vec4 final_color;
        void main() {
          final_color = vec4(1.0, 0.5, 0.2, 1.0);
        }
        "#;

      gl::ShaderSource(frag_shader, 1, 
        &(FRAG_SHADER.as_bytes().as_ptr().cast()),
        &(FRAG_SHADER.len().try_into().unwrap()));
      
      gl::CompileShader(frag_shader);

      let mut success = 0;
      gl::GetShaderiv(frag_shader, gl::COMPILE_STATUS, &mut success);

      if success == 0 {
        let mut v: Vec<u8> = Vec::with_capacity(1024);
        let mut log_len = 0_i32;
        gl::GetShaderInfoLog(frag_shader, 1024, &mut log_len, 
          v.as_mut_ptr().cast());
        v.set_len(log_len.try_into().unwrap());
        panic!("Frag Shader Compile Error: {}", String::from_utf8_lossy(&v));
      }

      let shader_program = gl::CreateProgram();
      gl::AttachShader(shader_program, vertex_shader);
      gl::AttachShader(shader_program, frag_shader);
      gl::LinkProgram(shader_program);
      gl::DeleteShader(vertex_shader);
      gl::DeleteShader(frag_shader);
      gl::UseProgram(shader_program); 

      SDL_GL_SetSwapInterval(1);

      let egui = EguiIntegration::new();

      return GfxRuntime { 
        window, 
        gl_context, 
        vao,
        vbo, 
        egui, 
        shader_program,
        should_quit: false,
        events: Vec::new()
      };
    }
  }

  extern "system" fn debug_message_callback(source: GLenum, msg_type: GLenum, id: GLuint, severity: GLenum, length: GLsizei,
    message: *const i8, user_param: *mut c_void) {
      if severity == gl::DEBUG_SEVERITY_NOTIFICATION {
        return;
      }
      
      let severity_str = if severity == gl::DEBUG_SEVERITY_LOW {
        "message"
      } else if severity == gl::DEBUG_SEVERITY_MEDIUM {
        "warning"
      } else {
        "error"
      };

      unsafe { println!("OpenGL {}: {:?}", severity_str, CStr::from_ptr(message)); }
      assert!(severity != gl::DEBUG_SEVERITY_HIGH);
  }

  pub fn begin_frame(&mut self) {
    unsafe {
      self.events.clear();
      let mut event = SDL_Event::default();
      while SDL_PollEvent(&mut event) == 1 {
        if event.type_.0 == SDL_WINDOWEVENT.0 {
          if event.window.event.0 == SDL_WINDOWEVENT_CLOSE.0 {
            self.should_quit = true;
          }
        } else if event.type_.0 == SDL_MOUSEBUTTONDOWN.0
          || event.type_.0 == SDL_MOUSEBUTTONUP.0
          || event.type_.0 == SDL_MOUSEMOTION.0
          || event.type_.0 == SDL_KEYDOWN.0
          || event.type_.0 == SDL_KEYUP.0
          || event.type_.0 == SDL_TEXTINPUT.0 {
            self.events.push(event);
        }
      }

      gl::Clear(gl::COLOR_BUFFER_BIT);
    }
  }

  pub fn end_frame(&mut self, ui_callback: impl FnOnce(&Context)) {
    self.egui.run(self.window, &self.events, ui_callback);

    unsafe { SDL_GL_SwapWindow(self.window) }
  }

  pub fn should_quit(&self) -> bool {
    self.should_quit
  }

  pub fn get_egui(&mut self) -> &mut EguiIntegration {
    &mut self.egui
  }
}
