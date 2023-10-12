//! A humble little game engine.

/// Core engine features.
pub mod engine;

/// Low-level graphics utilities.
pub mod gfx;

/// Vector, matrix, and spatial math utilities.
pub mod math;

/// Editor implementation.
pub mod ed;

/// Asset system implementation.
pub mod ass;

/// A standard Result type used by various engine systems.
pub type Result<T> = std::result::Result<T, Error>;


/// A standard Error type used by various engine systems.
#[derive(Debug, Clone)]
pub struct Error {
  message: String
}

impl Error {
  fn new<T>(msg: &T) -> Self where T: ToString + ?Sized {
    return Self {
      message: msg.to_string()
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

#[cfg(test)]
mod tests {
  
}
