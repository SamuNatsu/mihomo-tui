use std::sync::{Mutex, OnceLock};

pub struct App {
    pub running: Mutex<bool>,
    pub help_text: Mutex<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: Mutex::new(true),
            help_text: Mutex::new("[ESC]Quit".into()),
        }
    }
}

impl App {
    pub fn get_instance() -> &'static App {
        static INSTANCE: OnceLock<App> = OnceLock::new();
        INSTANCE.get_or_init(App::default)
    }
}
