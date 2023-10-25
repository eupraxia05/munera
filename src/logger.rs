use std::borrow::BorrowMut;
use std::sync::Mutex;
use std::cell::RefCell;

#[derive(Clone)]
pub struct LoggerMessage {
  level: log::Level,
  message: String
}

impl LoggerMessage {
  fn new(record: &log::Record) -> Self {
    Self {
      level: record.level(),
      message: format!("[{}] {}", record.module_path().unwrap_or_default(), record.args().to_string())
    }
  }

  pub fn level(&self) -> log::Level {
    self.level
  }

  pub fn message(&self) -> &String {
    &self.message
  }
}

pub struct Logger {
  pub messages: Mutex<Vec<LoggerMessage>>
}

impl log::Log for Logger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    true // !metadata.target().starts_with("wgpu") || metadata.level() <= log::Level::Warn
  }

  fn log(&self, record: &log::Record) {
    if !self.enabled(record.metadata()) {
      return
    }

    let msg = format!("[{}] {}: {}", record.module_path().unwrap_or_default(), record.level(), record.args());
    println!("{}", msg);
    let mut lock = self.messages.lock().expect("Failed to lock logger messages!");
    lock.push(LoggerMessage::new(record));
  }

  fn flush (&self) {

  }
}

pub static LOGGER: Logger = Logger { 
  messages: Mutex::new(Vec::new())
};
