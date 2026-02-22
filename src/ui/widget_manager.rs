use crate::ipc::WidgetKind;
use masonry::core::WidgetId;
use masonry::core::WidgetTag;
use masonry::widgets::Flex;
use std::collections::HashMap;

/// Tag for the root Flex container that holds all dynamically created widgets.
pub const ROOT_FLEX_TAG: WidgetTag<Flex> = WidgetTag::new("root_flex");

/// Information tracked for each JS-created widget.
#[derive(Debug, Clone)]
pub struct WidgetInfo {
    /// The masonry WidgetId assigned when the widget was inserted.
    pub widget_id: WidgetId,
    /// What kind of widget this is.
    pub kind: WidgetKind,
    /// The parent widget JS id (None means root Flex).
    pub parent_id: Option<String>,
    /// Index in the parent Flex's children list.
    pub child_index: usize,
}

/// Manages the mapping from JS widget IDs to masonry widget state.
pub struct WidgetManager {
    /// Maps JS string IDs → tracked widget info.
    pub widgets: HashMap<String, WidgetInfo>,
    /// Tracks how many children each Flex container has (by JS id, or "__root__" for root).
    pub child_counts: HashMap<String, usize>,
}

impl WidgetManager {
    pub fn new() -> Self {
        let mut child_counts = HashMap::new();
        child_counts.insert("__root__".to_string(), 0);
        Self {
            widgets: HashMap::new(),
            child_counts,
        }
    }

    fn parent_matches(info: &WidgetInfo, parent_key: &str) -> bool {
        match info.parent_id.as_deref() {
            Some(pid) => pid == parent_key,
            None => parent_key == "__root__",
        }
    }

    pub fn current_child_count(&self, parent_key: &str) -> usize {
        self.widgets
            .values()
            .filter(|info| Self::parent_matches(info, parent_key))
            .count()
    }

    pub fn next_child_index(&mut self, parent_key: &str) -> usize {
        let idx = self.current_child_count(parent_key);
        self.child_counts.insert(parent_key.to_string(), idx + 1);
        idx
    }

    fn collect_descendants(&self, parent_id: &str, out: &mut Vec<String>) {
        let children: Vec<String> = self
            .widgets
            .iter()
            .filter_map(|(id, info)| {
                if info.parent_id.as_deref() == Some(parent_id) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        for child_id in children {
            out.push(child_id.clone());
            self.collect_descendants(&child_id, out);
        }
    }

    fn recompute_parent_state(&mut self, parent_key: &str) {
        let mut child_ids: Vec<String> = self
            .widgets
            .iter()
            .filter_map(|(id, info)| {
                if Self::parent_matches(info, parent_key) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        child_ids.sort_by_key(|id| self.widgets.get(id).map(|w| w.child_index).unwrap_or(usize::MAX));

        for (new_index, child_id) in child_ids.iter().enumerate() {
            if let Some(info) = self.widgets.get_mut(child_id) {
                info.child_index = new_index;
            }
        }

        self.child_counts.insert(parent_key.to_string(), child_ids.len());
    }

    pub fn remove_widget_subtree(&mut self, id: &str) -> Option<WidgetInfo> {
        let removed = self.widgets.remove(id)?;

        let mut descendants = Vec::new();
        self.collect_descendants(id, &mut descendants);
        for child_id in descendants {
            self.widgets.remove(&child_id);
            self.child_counts.remove(&child_id);
        }

        self.child_counts.remove(id);

        let parent_key = removed.parent_id.as_deref().unwrap_or("__root__").to_string();
        self.recompute_parent_state(&parent_key);

        Some(removed)
    }
}
