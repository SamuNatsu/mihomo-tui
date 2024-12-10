use std::sync::{Mutex, OnceLock};

use chrono::Local;

pub struct Logger {
    buffer: Vec<(LogLevel, String)>,
    buffer_size: usize,
}

impl Logger {
    pub fn get_instance() -> &'static Mutex<Logger> {
        static INSTANCE: OnceLock<Mutex<Logger>> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Mutex::new(Self {
                buffer: Vec::new(),
                buffer_size: 1_000,
            })
        })
    }

    pub fn set_buffer_size(&mut self, new_size: usize) {
        self.buffer_size = new_size;

        let buffer_len = self.buffer.len();
        if buffer_len > new_size {
            self.buffer.drain(0..(buffer_len - new_size));
        }
    }
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn get_buffer(&self) -> &Vec<(LogLevel, String)> {
        &self.buffer
    }

    pub fn trace<S>(&mut self, text: S)
    where
        S: Into<String>,
    {
        self.push(LogLevel::Trace, text.into());
    }

    pub fn debug<S>(&mut self, text: S)
    where
        S: Into<String>,
    {
        self.push(LogLevel::Debug, text.into());
    }

    pub fn info<S>(&mut self, text: S)
    where
        S: Into<String>,
    {
        self.push(LogLevel::Info, text.into());
    }

    pub fn warn<S>(&mut self, text: S)
    where
        S: Into<String>,
    {
        self.push(LogLevel::Warn, text.into());
    }

    pub fn error<S>(&mut self, text: S)
    where
        S: Into<String>,
    {
        self.push(LogLevel::Error, text.into());
    }

    fn push(&mut self, log_level: LogLevel, text: String) {
        let text = format!(
            "[{}] [{}] {}",
            Local::now().format("%Y-%m-%dT%H%M%S"),
            log_level.to_str(),
            text
        );

        self.buffer.push((log_level, text));
        if self.buffer.len() > self.buffer_size {
            self.buffer.drain(0..1);
        }
    }
}

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Trace => "Trace",
            Self::Debug => "Debug",
            Self::Info => " Info",
            Self::Warn => " Warn",
            Self::Error => "Error",
        }
    }
}
