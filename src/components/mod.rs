mod profiles;
mod status;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use profiles::Profile;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame,
};
use status::Status;

use crate::app::App;

pub trait Component {
    fn new() -> Self;
    fn render(&self, area: &Rect, frame: &mut Frame);
    async fn tick(&mut self) -> Result<()>;
    async fn handle_event(&mut self, ev: &Event) -> Result<()>;
}

pub struct Root {
    main_component: RootMainComponent,
}

impl Component for Root {
    fn new() -> Self {
        Self {
            main_component: RootMainComponent::Status(Status::new()),
        }
    }

    fn render(&self, _: &Rect, frame: &mut Frame) {
        let [tabs_area, main_area, help_area] = Layout::vertical(vec![
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        frame.render_widget(self.create_tabs(), tabs_area);
        frame.render_widget(self.create_help(), help_area);

        match &self.main_component {
            RootMainComponent::Profiles(c) => c.render(&main_area, frame),
            _ => (),
        }
    }

    async fn tick(&mut self) -> Result<()> {
        match &mut self.main_component {
            RootMainComponent::Profiles(c) => c.tick().await?,
            _ => (),
        }

        Ok(())
    }

    async fn handle_event(&mut self, ev: &Event) -> Result<()> {
        match ev {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Esc => *App::get_instance().running.lock().unwrap() = false,
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            *App::get_instance().running.lock().unwrap() = false;
                        }
                    }
                    KeyCode::F(f) => match f {
                        1 => {
                            if self.main_component.as_usize() != 0 {
                                self.main_component = RootMainComponent::Status(Status::new());
                            }
                        }
                        2 => {
                            if self.main_component.as_usize() != 1 {
                                self.main_component = RootMainComponent::Profiles(Profile::new());
                            }
                        }
                        3 => (),
                        4 => (),
                        5 => (),
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }

        match &mut self.main_component {
            RootMainComponent::Profiles(c) => c.handle_event(ev).await?,
            _ => (),
        }

        Ok(())
    }
}

impl Root {
    fn create_tabs(&self) -> Tabs {
        Tabs::new(vec![
            "[F1]Status",
            "[F2]Profiles",
            "[F3]Proxies",
            "[F4]Rules",
            "[F5]Settings",
        ])
        .block(
            Block::new()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Thick),
        )
        .style(Style::default().dark_gray())
        .highlight_style(Style::default().light_yellow())
        .select(self.main_component.as_usize())
        .divider("")
    }

    fn create_help(&self) -> Paragraph {
        Paragraph::new(App::get_instance().help_text.lock().unwrap().clone())
            .on_white()
            .black()
            .bold()
    }
}

enum RootMainComponent {
    Status(Status),
    Profiles(Profile),
    Proxies,
    Rules,
    Settings,
}

impl RootMainComponent {
    pub const fn as_usize(&self) -> usize {
        match self {
            Self::Status(_) => 0,
            Self::Profiles(_) => 1,
            Self::Proxies => 2,
            Self::Rules => 3,
            Self::Settings => 4,
        }
    }
}
