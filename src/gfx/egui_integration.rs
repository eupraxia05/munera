use bytemuck::offset_of;
use egui::epaint::{ImageDelta, Primitive, Vertex};
use egui::epaint::textures::TextureFilter;
use egui::{RawInput, Rect, Vec2, Modifiers, Context, TextureId, ImageData, Color32, ClippedPrimitive, PaintCallbackInfo, Event, Pos2, PointerButton, TextureOptions};
use fermium::prelude::{SDL_MOUSEBUTTONDOWN, SDL_MOUSEBUTTONUP, SDL_MOUSEMOTION, SDL_Keycode, SDL_Keysym};
use fermium::scancode::SDL_SCANCODE_LEFT;
use crate::gfx::GfxRuntime;
use crate::math::{Vec2i, Vec2f, Vec3f};
use fermium::video::*;
use fermium::*;
extern crate gl;
use gl::types::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::size_of;
use fermium::events::*;
use fermium::mouse::*;
use fermium::scancode::*;
use fermium::prelude::*;
use egui::Key;
use std::ffi::CStr;
use std::ops::FnMut;

pub struct EguiIntegration {
  ctx: Context,
  textures: HashMap<TextureId, GLuint>,
  program: GLuint,
  u_screen_size: GLint,
  u_sampler: GLint,
  a_pos: GLint,
  a_tex_coord: GLint,
  a_tint: GLint,
  vao: GLuint,
  vbo: GLuint,
  element_array_buffer: GLuint,
  next_native_texture_id: u64
}

impl EguiIntegration {
  pub fn new() -> Self {
    let vert_shader = compile_shader(gl::VERTEX_SHADER, include_str!("../../shd/egui.vert"));
    let frag_shader = compile_shader(gl::FRAGMENT_SHADER, include_str!("../../shd/egui.frag"));
    let program = link_program(vert_shader, frag_shader);

    unsafe {
      gl::DetachShader(program, vert_shader);
      gl::DetachShader(program, frag_shader);
      gl::DeleteShader(vert_shader);
      gl::DeleteShader(frag_shader);

      let u_screen_size = gl::GetUniformLocation(program, b"u_screen_size\0".as_ptr() as *const i8);
      let u_sampler = gl::GetUniformLocation(program, b"u_sampler\0".as_ptr() as *const i8);
      let a_pos = gl::GetAttribLocation(program, b"a_pos\0".as_ptr() as *const i8);
      let a_tex_coord = gl::GetAttribLocation(program, b"a_tex_coord\0".as_ptr() as *const i8);
      let a_tint = gl::GetAttribLocation(program, b"a_tint\0".as_ptr() as *const i8);

      let mut vbo: GLuint = 0;
      gl::CreateBuffers(1, &mut vbo);

      let mut vao: GLuint = 0;
      gl::CreateVertexArrays(1, &mut vao);

      gl::BindVertexArray(vao);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

      let stride = std::mem::size_of::<Vertex>() as i32;

      gl::VertexAttribPointer(a_pos as GLuint, 2, gl::FLOAT, gl::FALSE, stride, offset_of!(Vertex, pos) as *const c_void);
      gl::VertexAttribPointer(a_tex_coord as GLuint, 2, gl::FLOAT, gl::FALSE, stride, 
        offset_of!(Vertex, uv) as *const c_void);
      gl::VertexAttribPointer(a_tint as GLuint, 4, gl::UNSIGNED_BYTE, gl::FALSE, stride, 
        offset_of!(Vertex, color) as *const c_void);
      
      gl::EnableVertexAttribArray(0);
      gl::EnableVertexAttribArray(1);
      gl::EnableVertexAttribArray(2);

      let mut element_array_buffer : GLuint = 0;
      gl::CreateBuffers(1, &mut element_array_buffer);

      assert!(gl::GetError() == gl::NO_ERROR);

      return Self { 
        ctx: Context::default(), 
        textures: HashMap::new(), 
        program: program, 
        u_screen_size: u_screen_size, 
        u_sampler: u_sampler, 
        a_pos: a_pos, 
        a_tex_coord: a_tex_coord,
        a_tint: a_tint,
        vao: vao,
        vbo: vbo,
        element_array_buffer: element_array_buffer,
        next_native_texture_id: 0
      };
    }
  }

  fn translate_sdl_keysym(key: SDL_Keysym) -> Option<Key> {   
    Some(match key.scancode {
      SDL_SCANCODE_DOWN => Key::ArrowDown,
      SDL_SCANCODE_LEFT => Key::ArrowLeft,
      SDL_SCANCODE_RIGHT => Key::ArrowRight,
      SDL_SCANCODE_UP => Key::ArrowUp,
      SDL_SCANCODE_ESCAPE => Key::Escape,
      SDL_SCANCODE_TAB => Key::Tab,
      SDL_SCANCODE_BACKSPACE => Key::Backspace,
      SDL_SCANCODE_TAB => Key::Tab,
      SDL_SCANCODE_RETURN => Key::Enter,
      SDL_SCANCODE_SPACE => Key::Space,
      SDL_SCANCODE_INSERT => Key::Insert,
      SDL_SCANCODE_DELETE => Key::Delete,
      SDL_SCANCODE_HOME => Key::Home,
      SDL_SCANCODE_END => Key::End,
      SDL_SCANCODE_PAGEUP => Key::PageUp,
      SDL_SCANCODE_PAGEDOWN => Key::PageDown,
      SDL_SCANCODE_MINUS => Key::Minus,
      SDL_SCANCODE_EQUALS => Key::PlusEquals,
      SDL_SCANCODE_KP_0 => Key::Num0,
      SDL_SCANCODE_KP_1 => Key::Num1,
      SDL_SCANCODE_KP_2 => Key::Num2,
      SDL_SCANCODE_KP_3 => Key::Num3,
      SDL_SCANCODE_KP_4 => Key::Num4,
      SDL_SCANCODE_KP_5 => Key::Num5,
      SDL_SCANCODE_KP_6 => Key::Num6,
      SDL_SCANCODE_KP_7 => Key::Num7,
      SDL_SCANCODE_KP_8 => Key::Num8,
      SDL_SCANCODE_KP_9 => Key::Num9,
      SDL_SCANCODE_A => Key::A,
      SDL_SCANCODE_B => Key::B,
      SDL_SCANCODE_C => Key::C,
      SDL_SCANCODE_D => Key::D,
      SDL_SCANCODE_E => Key::E,
      SDL_SCANCODE_F => Key::F,
      SDL_SCANCODE_G => Key::G,
      SDL_SCANCODE_H => Key::H,
      SDL_SCANCODE_I => Key::I,
      SDL_SCANCODE_J => Key::J,
      SDL_SCANCODE_K => Key::K,
      SDL_SCANCODE_L => Key::L,
      SDL_SCANCODE_M => Key::M,
      SDL_SCANCODE_N => Key::N,
      SDL_SCANCODE_O => Key::O,
      SDL_SCANCODE_P => Key::P,
      SDL_SCANCODE_Q => Key::Q,
      SDL_SCANCODE_R => Key::R,
      SDL_SCANCODE_S => Key::S,
      SDL_SCANCODE_T => Key::T,
      SDL_SCANCODE_U => Key::U,
      SDL_SCANCODE_V => Key::V,
      SDL_SCANCODE_W => Key::W,
      SDL_SCANCODE_X => Key::X,
      SDL_SCANCODE_Y => Key::Y,
      SDL_SCANCODE_Z => Key::Z,
      SDL_SCANCODE_F1 => Key::F1,
      SDL_SCANCODE_F2 => Key::F2,
      SDL_SCANCODE_F3 => Key::F3,
      SDL_SCANCODE_F4 => Key::F4,
      SDL_SCANCODE_F5 => Key::F5,
      SDL_SCANCODE_F6 => Key::F6,
      SDL_SCANCODE_F7 => Key::F7,
      SDL_SCANCODE_F8 => Key::F8,
      SDL_SCANCODE_F9 => Key::F9,
      SDL_SCANCODE_F10 => Key::F10,
      SDL_SCANCODE_F11 => Key::F11,
      SDL_SCANCODE_F12 => Key::F12,
      SDL_SCANCODE_F13 => Key::F13,
      SDL_SCANCODE_F14 => Key::F14,
      SDL_SCANCODE_F15 => Key::F15,
      SDL_SCANCODE_F16 => Key::F16,
      SDL_SCANCODE_F17 => Key::F17,
      SDL_SCANCODE_F18 => Key::F18,
      SDL_SCANCODE_F19 => Key::F19,
      SDL_SCANCODE_F20 => Key::F20,
      _ => {
        return None;
      }
    })
  }

  pub fn run(&mut self, window: *mut SDL_Window, events: &Vec<SDL_Event>, ui_callback: impl FnOnce(&Context)) {
    let mut raw_input = RawInput::default();
    let mut w : c_int = 0;
    let mut h : c_int = 0;
    let mut x : c_int = 0;
    let mut y : c_int = 0;
    unsafe { 
      SDL_GetWindowSize(window, &mut w, &mut h); 
      SDL_GetWindowPosition(window, &mut x, &mut y) 
    }
    raw_input.screen_rect = Some(Rect::from_min_size(Default::default(), 
      Vec2::new(w as f32, h as f32)));
    unsafe { raw_input.focused = 
      (SDL_GetWindowFlags(window) & SDL_WINDOW_INPUT_FOCUS.0 as u32) != 0; }
    
    let mut mouse_x : c_int = 0;
    let mut mouse_y : c_int = 0;
    unsafe { 
      SDL_GetMouseState(&mut mouse_x, &mut mouse_y);
    
      for event in events {
        if event.type_.0 == SDL_MOUSEBUTTONDOWN.0 || event.type_.0 == SDL_MOUSEBUTTONUP.0 {
          let mut pointer_button = PointerButton::Primary;
          if event.button.button == SDL_BUTTON_RIGHT as u8 {
            pointer_button = PointerButton::Secondary;
          } else if event.button.button == SDL_BUTTON_MIDDLE as u8 {
            pointer_button = PointerButton::Middle;
          }
          let pos = Pos2::new(event.button.x as f32, event.button.y as f32);
          let ev = Event::PointerButton { pos: pos, 
            button: (pointer_button), pressed: (event.type_.0 == SDL_MOUSEBUTTONDOWN.0), modifiers: (Modifiers::NONE) };
          raw_input.events.push(ev);
        } else if event.type_.0 == SDL_MOUSEMOTION.0 {
          raw_input.events.push(Event::PointerMoved(Pos2::new(event.motion.x as f32, event.motion.y as f32)));
        } else if event.type_.0 == SDL_KEYDOWN.0 || event.type_.0 == SDL_KEYUP.0 {
          let key = Self::translate_sdl_keysym(event.key.keysym);
          if key.is_some() {
            let modifiers = Modifiers {
              alt: (event.key.keysym.mod_ as i32 & KMOD_LALT.0 == KMOD_LALT.0)
                || (event.key.keysym.mod_ as i32 & KMOD_RALT.0 == KMOD_RALT.0),
              ctrl: (event.key.keysym.mod_ as i32 & KMOD_LCTRL.0 == KMOD_LCTRL.0)
                || (event.key.keysym.mod_ as i32 & KMOD_RCTRL.0 == KMOD_CTRL.0),
              shift: (event.key.keysym.mod_ as i32 & KMOD_LSHIFT.0 == KMOD_LSHIFT.0)
                || (event.key.keysym.mod_ as i32 & KMOD_RSHIFT.0 == KMOD_RSHIFT.0),
              mac_cmd: (event.key.keysym.mod_ as i32 & KMOD_LGUI.0 == KMOD_LGUI.0)
                || (event.key.keysym.mod_ as i32 & KMOD_RGUI.0 == KMOD_RGUI.0),
              command: (event.key.keysym.mod_ as i32 & KMOD_LCTRL.0 == KMOD_LCTRL.0)
                || (event.key.keysym.mod_ as i32 & KMOD_RCTRL.0 == KMOD_RCTRL.0),
            };
            raw_input.events.push(Event::Key { 
              key: key.unwrap(), pressed: event.type_.0 == SDL_KEYDOWN.0, 
              repeat: event.key.repeat != 0, modifiers: modifiers });
          }
        } else if event.type_.0 == SDL_TEXTINPUT.0 {
          let text = CStr::from_ptr(event.text.text.as_ptr());
          if let Ok(conv_text) = text.to_str() {
            raw_input.events.push(Event::Text(conv_text.to_string()));
          }
        }
      }
    }

    let full_output = self.ctx.run(raw_input, ui_callback);

    for delta in full_output.textures_delta.set {
      self.set_texture(delta.0, &delta.1)
    };

    self.paint_primitives(Vec2i::new(w, h), self.ctx.tessellate(full_output.shapes));

    for delta in full_output.textures_delta.free {
      self.free_texture(delta);
    }
  }

  fn set_texture(&mut self, tex_id: TextureId, delta: &ImageDelta) {
    let tex = self.textures.entry(tex_id).or_insert_with(|| unsafe 
    {
      let mut gltex: GLuint = 0;
      gl::CreateTextures(gl::TEXTURE_2D, 1, &mut gltex);
      assert!(gl::GetError() == gl::NO_ERROR);
      gltex
    });


    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, *tex);
      assert!(gl::GetError() == gl::NO_ERROR);
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
          gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr().cast::<c_void>());
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

  fn prepare_painting(&mut self, screen_size: &Vec2i) {
    unsafe {
      assert!(gl::GetError() == gl::NO_ERROR);
      gl::Enable(gl::SCISSOR_TEST);
      gl::Disable(gl::CULL_FACE);
      gl::Disable(gl::DEPTH_TEST);
      gl::ColorMask(255, 255, 255, 255);
      gl::Enable(gl::BLEND);
      gl::BlendEquationSeparate(gl::FUNC_ADD, gl::FUNC_ADD);
      gl::BlendFuncSeparate(gl::ONE, gl::ONE_MINUS_SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE);
      gl::Viewport(0, 0, screen_size.x, screen_size.y);
      gl::UseProgram(self.program);
      gl::Uniform2f(self.u_screen_size, screen_size.x as f32, screen_size.y as f32);
      gl::Uniform1i(self.u_sampler, 0);
      gl::ActiveTexture(gl::TEXTURE0);
      gl::BindVertexArray(self.vao);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.element_array_buffer);
      assert!(gl::GetError() == gl::NO_ERROR);
    }
  }

  fn paint_primitives(&mut self, screen_size: Vec2i, clipped_primitives: Vec<ClippedPrimitive>) {
    self.prepare_painting(&screen_size);

    for ClippedPrimitive { clip_rect, primitive } in clipped_primitives {
      let clip_min_x = (clip_rect.min.x as i32).clamp(0, screen_size.x);
      let clip_min_y = (clip_rect.min.y as i32).clamp(0, screen_size.y);
      let clip_max_x = (clip_rect.max.x as i32).clamp(clip_min_x, screen_size.x);
      let clip_max_y = (clip_rect.max.y as i32).clamp(clip_min_y, screen_size.y);

      unsafe {
        assert!(gl::GetError() == gl::NO_ERROR);
        gl::Scissor(clip_min_x, screen_size.y - clip_max_y, 
          clip_max_x - clip_min_x, clip_max_y - clip_min_y);
        assert!(gl::GetError() == gl::NO_ERROR);
      }

      match primitive {
        Primitive::Mesh(mesh) => {
          if let Some(texture) = self.textures.get(&mesh.texture_id) {
            unsafe {
              assert!(gl::GetError() == gl::NO_ERROR);
              gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
              gl::BufferData(gl::ARRAY_BUFFER, mesh.vertices.len() as isize * size_of::<Vertex>() as isize, 
                mesh.vertices.as_ptr().cast::<c_void>(), gl::STREAM_DRAW);
              gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.element_array_buffer);
              gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, mesh.indices.len() as isize * size_of::<u32>() as isize, 
                mesh.indices.as_ptr().cast::<c_void>(), gl::STREAM_DRAW);
              gl::BindTexture(gl::TEXTURE_2D, *texture);
              gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null());
              assert!(gl::GetError() == gl::NO_ERROR);
            }
          }
        }
        Primitive::Callback(callback) => {
          if callback.rect.is_positive() {
            unsafe {
              gl::Viewport(callback.rect.min.x.round() as i32, 
                screen_size.y - callback.rect.max.y.round() as i32,
                callback.rect.max.x.round() as i32 - callback.rect.min.x.round() as i32, 
                callback.rect.max.y.round() as i32 - callback.rect.min.y.round() as i32);

              let info = PaintCallbackInfo {
                viewport: callback.rect,
                clip_rect: clip_rect,
                pixels_per_point: 1.0f32,
                screen_size_px: [screen_size.x as u32, screen_size.y as u32]
              };

              if let Some(callback) = callback.callback.downcast_ref::<CallbackFn>() {
                (callback.f)(info, self);
              }
            }

            self.prepare_painting(&screen_size);
          }
        }
      }
    }

    unsafe {
      gl::BindVertexArray(0);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
      gl::Disable(gl::SCISSOR_TEST);

      assert!(gl::GetError() == gl::NO_ERROR);
    }
  }

  pub fn register_native_texture(&mut self, tex: GLuint) -> egui::TextureId 
  {
    let id = egui::TextureId::User(self.next_native_texture_id);
    self.next_native_texture_id += 1;
    self.textures.insert(id, tex);
    id
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

pub struct CallbackFn {
  f: Box<dyn Fn(PaintCallbackInfo, &EguiIntegration) + Sync + Send>,
}
