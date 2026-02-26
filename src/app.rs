//! Terminal user interface.

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};

use crate::fs_tree::FsTree;

use self::file_list_widget::FileListWidget;

mod file_list_widget;

pub struct App {
    file_list: FileListWidget,
    fs_tree: FsTree,
    running: bool,
}

impl App {
    pub fn new(fs_tree: FsTree) -> Self {
        Self {
            file_list: FileListWidget::new(&fs_tree),
            fs_tree,
            running: false,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        self.file_list.render(frame);
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Down) => self.file_list.next(),
            (_, KeyCode::Up) => self.file_list.prev(),
            (_, KeyCode::Left) => self.file_list.back(&self.fs_tree),
            (_, KeyCode::Right) => self.file_list.enter(&self.fs_tree),
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}
