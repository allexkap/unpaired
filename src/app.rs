//! Terminal user interface.
//!
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
    widgets::Block,
};

use crate::{
    app::fs_tree_panel::{FsTreePanel, FsTreePanelState},
    fs_tree::FsTree,
};

mod fs_tree_panel;

pub struct App {
    panel_state: FsTreePanelState,
    fs_tree: FsTree,
    running: bool,
}

impl App {
    pub fn new(fs_tree: FsTree) -> Self {
        Self {
            panel_state: FsTreePanelState::new(&fs_tree),
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
        let area = frame.area();

        let [header, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(body);

        frame.render_widget(
            Line::styled(
                format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
                Style::new().reversed(),
            ),
            header,
        );

        frame.render_widget(Line::styled("Placeholer", Style::new().reversed()), footer);

        frame.render_stateful_widget(FsTreePanel::default(), left, &mut self.panel_state);

        frame.render_widget(Block::bordered(), right);
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
            (_, KeyCode::Down) => self.panel_state.next(),
            (_, KeyCode::Up) => self.panel_state.prev(),
            (_, KeyCode::Left) => self.panel_state.back(&self.fs_tree),
            (_, KeyCode::Right) => self.panel_state.enter(&self.fs_tree),
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}
