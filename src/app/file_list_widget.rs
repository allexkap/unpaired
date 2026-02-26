//! File list widget for browsing a single tree level.

use ratatui::{
    Frame,
    widgets::{HighlightSpacing, List, ListState},
};

use crate::fs_tree::{FsTree, FsTreeNodeId};

#[derive(Default)]
pub struct FileListWidget {
    current: Option<FsTreeNodeId>,
    items: Vec<(FsTreeNodeId, String)>,
    state: ListState,
}

impl FileListWidget {
    pub fn new(fs_tree: &FsTree) -> Self {
        let mut new_self = Self::default();
        new_self.update(None, fs_tree);
        new_self
    }

    pub fn next(&mut self) {
        self.state.select_next();
    }

    pub fn prev(&mut self) {
        self.state.select_previous();
    }

    pub fn back(&mut self, fs_tree: &FsTree) {
        self.update(self.current.and_then(|id| fs_tree.get_parent(id)), fs_tree);
    }

    pub fn enter(&mut self, fs_tree: &FsTree) {
        let Some(pos) = self.state.selected() else {
            return;
        };
        let selected_node_id = self.items[pos].0;
        if fs_tree.get_node(selected_node_id).kind.is_dir() {
            self.update(Some(selected_node_id), fs_tree);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let list = List::new(self.items.iter().map(|i| i.1.clone()))
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, frame.area(), &mut self.state);
    }

    fn update(&mut self, node_id: Option<FsTreeNodeId>, fs_tree: &FsTree) {
        self.current = node_id;
        self.items = match node_id {
            Some(id) => fs_tree
                .get_children(id)
                .into_iter()
                .map(|child_id| {
                    let name = format!(
                        "{} {}",
                        fs_tree.get_name(child_id).to_owned(),
                        fs_tree
                            .get_same_nodes(child_id)
                            .map(|g| g.len())
                            .unwrap_or(0)
                    );
                    (child_id, name)
                })
                .collect(),
            None => fs_tree.get_roots(),
        };
        if self.items.len() > 0 {
            self.state.select_first();
        };
    }
}
