//! Terminal user interface.

use std::{cell::RefCell, rc::Rc};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Style,
    text::Line,
};

use crate::{
    app::components::{Component, FsTreePanel, SameNodesPanel},
    fs_tree::FsTree,
};

mod components;

const APP_HELP: &'static str = "WARNING: Work in Progress";

pub struct App {
    main_panel: FsTreePanel,
    info_panel: SameNodesPanel,
    fs_tree: Rc<RefCell<FsTree>>,
    running: bool,
}

impl App {
    pub fn new(fs_tree: FsTree) -> Self {
        let fs_tree_ref = Rc::new(RefCell::new(fs_tree));
        let main_panel = FsTreePanel::new(fs_tree_ref.clone());
        let info_panel =
            SameNodesPanel::new(main_panel.get_selected().unwrap(), &fs_tree_ref.borrow());

        Self {
            main_panel,
            info_panel,
            fs_tree: fs_tree_ref.clone(),
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
                format!(
                    "{} v{} ~ {}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                    APP_HELP
                ),
                Style::new().reversed(),
            ),
            header,
        );

        frame.render_widget(Line::styled("Placeholer", Style::new().reversed()), footer);

        self.main_panel.render(frame, left);

        self.info_panel.render(frame, right);
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
            _ => self.main_panel.handle_key_event(key).unwrap(),
        }
        // todo
        self.info_panel = SameNodesPanel::new(
            self.main_panel.get_selected().unwrap(),
            &self.fs_tree.borrow(),
        );
    }

    fn quit(&mut self) {
        self.running = false;
    }
}
