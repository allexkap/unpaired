use ratatui::{
    layout::{Constraint, Layout},
    text::Span,
    widgets::{HighlightSpacing, List, ListState, StatefulWidget, Widget},
};

use crate::fs_tree::{FsTree, FsTreeNodeId};

#[derive(Default)]
pub struct FsTreePanelState {
    current: Option<(FsTreeNodeId, String)>,
    items: Vec<(FsTreeNodeId, String)>,
    list_state: ListState,
}

impl FsTreePanelState {
    pub fn new(fs_tree: &FsTree) -> Self {
        let mut new_self = Self::default();
        let roots = fs_tree.get_roots();
        let start_node_id = if roots.len() == 1 {
            Some(roots[0].0)
        } else {
            None
        };
        new_self.update(start_node_id, fs_tree);
        new_self
    }

    pub fn next(&mut self) {
        self.list_state.select_next();
    }

    pub fn prev(&mut self) {
        self.list_state.select_previous();
    }

    pub fn back(&mut self, fs_tree: &FsTree) {
        self.update(
            self.current
                .as_ref()
                .and_then(|(id, _)| fs_tree.get_parent(*id)),
            fs_tree,
        );
    }

    pub fn enter(&mut self, fs_tree: &FsTree) {
        let Some(pos) = self.list_state.selected() else {
            return;
        };
        let selected_node_id = self.items[pos].0;
        if fs_tree.get_node(selected_node_id).kind.is_dir() {
            self.update(Some(selected_node_id), fs_tree);
        }
    }

    fn update(&mut self, node_id: Option<FsTreeNodeId>, fs_tree: &FsTree) {
        match node_id {
            Some(id) => {
                self.current = Some((id, fs_tree.get_full_path(id).to_string_lossy().to_string()));
                self.items = fs_tree
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
                    .collect();
            }
            None => {
                let roots = fs_tree.get_roots();
                if roots.len() == 1 {
                    self.update(Some(roots[0].0), fs_tree);
                    return;
                }
                self.current = None;
                self.items = roots;
            }
        };
        self.list_state.select_first()
    }
}

pub struct FsTreePanel;

impl Default for FsTreePanel {
    fn default() -> Self {
        Self
    }
}

impl StatefulWidget for FsTreePanel {
    type State = FsTreePanelState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let [header, content] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let text = match &state.current {
            Some((_, path)) => path.as_str(),
            None => "Select root",
        };

        Widget::render(Span::raw(text), header, buf);

        StatefulWidget::render(
            List::new(state.items.iter().map(|i| i.1.as_str()))
                .highlight_symbol("> ")
                .highlight_spacing(HighlightSpacing::Always),
            content,
            buf,
            &mut state.list_state,
        );
    }
}
