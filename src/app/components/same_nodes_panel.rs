use std::{cell::RefCell, rc::Rc};

use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{HighlightSpacing, List, ListState},
};

use crate::fs_tree::{FileGroup, FsTree, FsTreeNodeId};

use super::Component;

pub struct SameNodesPanel {
    fs_tree: Rc<RefCell<FsTree>>,
    node_id: Option<FsTreeNodeId>,
    list_state: ListState,
}

impl SameNodesPanel {
    pub fn new(node_id: Option<FsTreeNodeId>, fs_tree: Rc<RefCell<FsTree>>) -> Self {
        Self {
            fs_tree,
            node_id,
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
        let fs_tree = self.fs_tree.borrow();
        let items = self
            .node_id
            .and_then(|node_id| fs_tree.get_same_nodes(node_id))
            .and_then(|group| match group {
                FileGroup::Duplicates(node_ids) => Some(
                    node_ids
                        .iter()
                        .map(|&node_id| format!("{:?}", fs_tree.get_full_path(node_id)))
                        .take(area.height as usize)
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .unwrap_or_else(|| Vec::new());

        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}
