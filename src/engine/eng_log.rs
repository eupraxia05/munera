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
      message: record.args().to_string()
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
    true
  }

  fn log(&self, record: &log::Record) {
    let msg = format!("{}: {}", record.level(), record.args());
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
