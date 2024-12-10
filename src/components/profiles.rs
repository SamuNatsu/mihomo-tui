use std::cell::RefCell;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::{app::App, config::profile::ProfileManager, utils::logger::{LogLevel, Logger}};

use super::Component;

pub struct Profile {
    table_state: RefCell<TableState>,
}

impl Component for Profile {
    fn new() -> Self {
        *App::get_instance().help_text.lock().unwrap() =
            "[ESC]Quit  [UP/DOWN]Move cursor  [ENTER]Activate  [A]Add  [D]Delete  [E]Edit  [U]Update".into();

        let mut table_state = TableState::new();
        table_state.select(Some(0));

        Self {
            table_state: RefCell::new(table_state),
        }
    }

    fn render(&self, area: &Rect, frame: &mut Frame) {
        let [table_area, log_area] =
            Layout::vertical(vec![Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)]).areas(*area);

        frame.render_stateful_widget(
            self.create_table(),
            table_area,
            &mut self.table_state.borrow_mut(),
        );
        frame.render_widget(self.create_log(log_area.height), log_area);
    }

    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    async fn handle_event(&mut self, ev: &Event) -> Result<()> {
        match ev {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Up => {
                        let mut state = self.table_state.borrow_mut();
                        let selected = state.selected().unwrap();

                        if selected > 0 {
                            state.select(Some(selected - 1));
                        }
                    }
                    KeyCode::Down => {
                        let total = ProfileManager::get_all().lock().unwrap().len();
                        let mut state = self.table_state.borrow_mut();
                        let selected = state.selected().unwrap();

                        if selected < total {
                            state.select(Some(selected + 1));
                        }
                    }
                    KeyCode::Enter => {
                        let selected = self.table_state.borrow().selected().unwrap();

                        if selected == 0 {
                            Logger::get_instance()
                                .lock()
                                .unwrap()
                                .info("Activating fallback profile");

                            if let Err(err) = ProfileManager::active_fallback_profile().await {
                                Logger::get_instance()
                                    .lock()
                                    .unwrap()
                                    .error(format!("{:#}", err));
                            }
                        } else {
                            let mut profiles = ProfileManager::get_all().lock().unwrap();
                            let profile = profiles.get_mut(selected - 1).unwrap();

                            Logger::get_instance()
                                .lock()
                                .unwrap()
                                .info(format!("Activating profile \"{}\"", profile.name));

                            if let Err(err) = profile.activate().await {
                                Logger::get_instance()
                                    .lock()
                                    .unwrap()
                                    .error(format!("{:#}", err));
                            }
                        }
                    }
                    KeyCode::Char('u') | KeyCode::Char('U') => {
                        let selected = self.table_state.borrow().selected().unwrap();

                        if selected != 0 {
                            ProfileManager::get_all()
                                .lock()
                                .unwrap()
                                .get_mut(selected - 1)
                                .unwrap()
                                .activate()
                                .await?;
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }

        Ok(())
    }
}

impl Profile {
    fn create_table(&self) -> Table {
        let header = Row::new(
            ["Active", "Name", "Type", "Updated At", "Used", "Expired At"]
                .into_iter()
                .map(|s| Cell::new(Text::from(s).centered()).on_blue())
                .collect::<Vec<Cell>>(),
        )
        .on_light_blue()
        .white()
        .bold();

        let mut rows = ProfileManager::get_all()
            .lock()
            .unwrap()
            .iter()
            .map(|p| {
                Row::new(vec![
                    Cell::default(),
                    if p.updating {
                        Cell::new(p.name.clone()).light_yellow()
                    } else {
                        Cell::new(p.name.clone())
                    },
                    if p.remote.is_none() {
                        Cell::new(Text::from("local").centered()).light_yellow()
                    } else {
                        Cell::new(Text::from("remote").centered()).light_green()
                    },
                    if p.remote.is_none() {
                        Cell::new(Text::from("N/A").centered()).dark_gray().italic()
                    } else {
                        if let Some(timestamp) = p.updated_at {
                            Cell::new(
                                Text::from(
                                    Utc.timestamp_opt(timestamp as i64, 0)
                                        .unwrap()
                                        .format("%Y-%m-%d")
                                        .to_string(),
                                )
                                .centered(),
                            )
                        } else {
                            Cell::new(Text::from("None").centered()).dark_gray()
                        }
                    },
                    if let Some(percent) = &p.get_used_str() {
                        Cell::new(Text::from(percent.clone()).centered())
                    } else {
                        Cell::new(Text::from("N/A").centered()).dark_gray().italic()
                    },
                    if let Some(timestamp) = p.expired_at {
                        Cell::new(
                            Text::from(
                                Utc.timestamp_opt(timestamp as i64, 0)
                                    .unwrap()
                                    .format("%Y-%m-%d")
                                    .to_string(),
                            )
                            .centered(),
                        )
                    } else {
                        Cell::new(Text::from("N/A").centered()).dark_gray().italic()
                    },
                ])
            })
            .collect::<Vec<Row>>();

        let fallback_profile = Row::new(vec![
            Cell::new(Text::from("X").centered()).green().bold(),
            Cell::new("Fallback"),
            Cell::new(Text::from("builtin").centered()).light_red(),
            Cell::new(Text::from("N/A").centered()).dark_gray().italic(),
            Cell::new(Text::from("N/A").centered()).dark_gray().italic(),
            Cell::new(Text::from("N/A").centered()).dark_gray().italic(),
        ]);
        rows.insert(0, fallback_profile);

        Table::new(
            rows,
            vec![
                Constraint::Length(8),
                Constraint::Min(4),
                Constraint::Length(9),
                Constraint::Length(12),
                Constraint::Length(10),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .row_highlight_style(Style::default().on_white().black())
    }

    fn create_log(&self, height: u16) -> Paragraph {
        let logger = Logger::get_instance().lock().unwrap();
        let lines = logger
            .get_buffer()
            .iter()
            .map(|(log_level, text)| match log_level {
                LogLevel::Trace => Line::from(text.clone()),
                LogLevel::Debug => Line::from(text.clone()).blue(),
                LogLevel::Info => Line::from(text.clone()).green(),
                LogLevel::Warn => Line::from(text.clone()).yellow(),
                LogLevel::Error => Line::from(text.clone()).red(),
            })
            .collect::<Vec<Line>>();
        let line_len = lines.len() as u16;

        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .title(" Log "),
            )
            .scroll((line_len.checked_sub(height).or(Some(0)).unwrap(), 0))
    }
}
