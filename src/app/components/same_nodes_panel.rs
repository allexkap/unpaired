use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{HighlightSpacing, List, ListState},
};

use crate::fs_tree::{FileGroup, FsTree, FsTreeNodeId};

use super::Component;

#[derive(Default)]
pub struct SameNodesPanel {
    paths: Vec<String>,
    list_state: ListState,
}

impl SameNodesPanel {
    pub fn new(node_id: FsTreeNodeId, fs_tree: &FsTree) -> Self {
        let paths = match fs_tree.get_same_nodes(node_id) {
            Some(FileGroup::Duplicates(node_ids)) => node_ids
                .iter()
                .map(|&node_id| format!("{:?}", fs_tree.get_full_path(node_id)))
                .collect(),
            _ => Vec::new(),
        };

        Self {
            paths,
            list_state: ListState::default(),
        }
    }
}

impl Component for SameNodesPanel {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let _ = key;
        todo!();
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items = self.paths.iter().map(|f| f.as_str());
        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}
