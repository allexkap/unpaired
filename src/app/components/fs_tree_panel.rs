use std::{cell::RefCell, rc::Rc};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Cell, HighlightSpacing, Row, Table, TableState},
};

use crate::{
    fs_tree::{FsTree, FsTreeNodeId, NodeKind},
    utils::bytes_to_string,
};

use super::Component;

struct FsTreeEntry {
    node_id: FsTreeNodeId,
    name: String,
    kind: NodeKind,
}

impl FsTreeEntry {
    const COLUMNS: usize = 6;
    const WIDTHS: [Constraint; Self::COLUMNS] = [
        Constraint::Max(4),  // size
        Constraint::Max(1),  //
        Constraint::Max(10), // dupes
        Constraint::Max(1),  //
        Constraint::Max(1),  // dir
        Constraint::Fill(1), // name
    ];

    fn new(node_id: FsTreeNodeId, fs_tree: &FsTree) -> Self {
        let node = fs_tree.get_node(node_id);

        Self {
            node_id,
            name: node.name.to_string_lossy().to_string(),
            kind: node.kind.clone(),
        }
    }

    fn to_row(&self) -> Row<'_> {
        let size = bytes_to_string(self.kind.get_total_size());
        let is_dir = if self.kind.is_dir() { "/" } else { " " };
        let (uniq, color) = match &self.kind {
            NodeKind::Dir(dir_node) => (
                format!("{}/{}", dir_node.unique_files_count, dir_node.files_count),
                if dir_node.unique_files_count == dir_node.files_count {
                    Color::Green
                } else if dir_node.unique_files_count == 0 {
                    Color::Red
                } else {
                    Color::Yellow
                },
            ),
            NodeKind::File(file_node) => (
                format!("{}", file_node.copies_count),
                if file_node.copies_count == 1 {
                    Color::Green
                } else {
                    Color::Red
                },
            ),
            NodeKind::Error(err) => (err.clone(), Color::default()),
        };

        let cells: [Cell; Self::COLUMNS] = [
            Line::raw(size).alignment(Alignment::Right).into(),
            Cell::default(),
            Line::styled(uniq, Style::default().fg(color))
                .alignment(Alignment::Right)
                .into(),
            Cell::default(),
            is_dir.into(),
            self.name.clone().into(),
        ];

        Row::new(cells)
    }
}

struct FsTreePanelState {
    current: (FsTreeNodeId, String),
    entries: Vec<FsTreeEntry>,
    widget_state: TableState,
}

impl FsTreePanelState {
    fn new(node_id: FsTreeNodeId, fs_tree: &FsTree) -> Self {
        let current = (
            node_id,
            fs_tree.get_full_path(node_id).to_string_lossy().to_string(),
        );

        let mut entries: Vec<FsTreeEntry> = fs_tree
            .get_children(node_id)
            .into_iter()
            .map(|child_id| FsTreeEntry::new(child_id, fs_tree))
            .collect();

        entries.sort_by(|a, b| b.kind.get_uniqueness().total_cmp(&a.kind.get_uniqueness()));

        let mut widget_state = TableState::default();
        widget_state.select_first();

        Self {
            current,
            entries,
            widget_state,
        }
    }

    fn next(&mut self) {
        self.widget_state.select_next();
    }

    fn prev(&mut self) {
        self.widget_state.select_previous();
    }

    fn enter(&self, fs_tree: &FsTree) -> Option<Self> {
        self.get_selected()
            .filter(|node_id| fs_tree.get_node(*node_id).kind.is_dir())
            .map(|node_id| Self::new(node_id, fs_tree))
    }

    fn get_selected(&self) -> Option<FsTreeNodeId> {
        self.widget_state
            .selected()
            .map(|pos| self.entries[pos].node_id)
    }
}

pub struct FsTreePanel {
    fs_tree: Rc<RefCell<FsTree>>,
    state: FsTreePanelState,
    stack: Vec<FsTreePanelState>,
}

impl FsTreePanel {
    pub fn new(fs_tree: Rc<RefCell<FsTree>>) -> Self {
        let node_id = fs_tree.borrow().get_roots().first().unwrap().0;

        let state = FsTreePanelState::new(node_id, &fs_tree.borrow());

        Self {
            fs_tree,
            state: state,
            stack: Vec::new(),
        }
    }

    pub fn get_selected(&self) -> Option<FsTreeNodeId> {
        self.state.get_selected()
    }

    fn enter(&mut self) {
        self.state.enter(&self.fs_tree.borrow()).map(|new_state| {
            self.stack
                .push(std::mem::replace(&mut self.state, new_state))
        });
    }

    fn back(&mut self) {
        if self.stack.len() > 0 {
            self.state = self.stack.pop().unwrap();
        }
    }
}

impl Component for FsTreePanel {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Down => self.state.next(),
            KeyCode::Up => self.state.prev(),
            KeyCode::Right => self.enter(),
            KeyCode::Left => self.back(),
            _ => {}
        };
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [header, content] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let header_text = &self.state.current.1;

        frame.render_widget(Span::raw(header_text), header);

        let rows: Vec<_> = self
            .state
            .entries
            .iter()
            .map(|entry| entry.to_row())
            .collect();

        let table = Table::new(rows, FsTreeEntry::WIDTHS)
            .column_spacing(0)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        self.state.widget_state.select_first_column();
        frame.render_stateful_widget(table, content, &mut self.state.widget_state);
    }
}
