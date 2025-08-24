use crate::app::{App, AppResult};
use crate::event::EventHandler;

use crate::ui::main_ui;
use ratatui::backend::Backend;
use ratatui::Terminal;

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal : Terminal<B>,
    pub events: EventHandler,
}

impl <B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self {terminal, events}
    }


    pub fn draw(&mut self, app: &mut App) -> AppResult<()>{
        self.terminal.draw(|frame| main_ui::render(app, frame))?;
        Ok(())
    }
}