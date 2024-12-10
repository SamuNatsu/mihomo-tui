use anyhow::Result;
use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};

use crate::app::App;

use super::Component;

pub struct Status {}

impl Component for Status {
    fn new() -> Self {
        *App::get_instance().help_text.lock().unwrap() = "[ESC]Quit".into();

        Self {}
    }

    fn render(&self, area: &Rect,  frame: &mut Frame) {}

    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    async fn handle_event(&mut self, ev: &Event) -> Result<()> {
        Ok(())
    }
}
