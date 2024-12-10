mod app;
mod components;
mod config;
mod event;
mod utils;

use std::panic;

use anyhow::Result;
use app::App;
use components::{Component, Root};
use event::{Event, EventHandler};

#[tokio::main]
async fn main() -> Result<()> {
    // Create terminal
    let mut terminal = ratatui::init();
    terminal.clear()?;

    // Set panic hook
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        ratatui::restore();
        panic_hook(panic);
    }));

    // Create event handler
    let mut event_handler = EventHandler::new(5);

    // Create root component
    let mut root = Root::new();

    // Run application
    while *App::get_instance().running.lock().unwrap() {
        terminal.draw(|frame| root.render(&frame.area(), frame))?;

        match event_handler.next().await? {
            Event::Tick => root.tick().await?,
            Event::Terminal(ev) => root.handle_event(&ev).await?,
        }
    }

    // Exit application
    ratatui::restore();
    Ok(())
}
