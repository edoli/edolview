use std::collections::HashSet;

#[derive(Default)]
pub(super) struct AssetListSelection {
    hashes: HashSet<String>,
    anchor: Option<String>,
}

impl AssetListSelection {
    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    pub fn contains(&self, hash: &str) -> bool {
        self.hashes.contains(hash)
    }

    pub fn hashes(&self) -> impl Iterator<Item = &String> {
        self.hashes.iter()
    }

    pub fn clear(&mut self) {
        self.hashes.clear();
        self.anchor = None;
    }

    pub fn activate(&mut self, hash: &str) {
        self.hashes.clear();
        self.anchor = Some(hash.to_owned());
    }

    pub fn select_range(&mut self, order: &[&str], target: &str, primary: Option<&str>, additive: bool) {
        let Some(end) = order.iter().position(|&hash| hash == target) else {
            return;
        };
        let start = self
            .anchor
            .as_deref()
            .or(primary)
            .and_then(|anchor| order.iter().position(|&hash| hash == anchor))
            .unwrap_or(end);
        self.anchor = Some(order[start].to_owned());
        if !additive {
            self.hashes.clear();
        }
        self.hashes
            .extend(order[start.min(end)..=start.max(end)].iter().map(|hash| (*hash).to_owned()));
    }

    pub fn retain(&mut self, mut exists: impl FnMut(&str) -> bool) {
        self.hashes.retain(|hash| exists(hash));
        if self.anchor.as_deref().is_some_and(|hash| !exists(hash)) {
            self.anchor = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_clears_selection_and_ranges_follow_reordered_rows() {
        let mut selection = AssetListSelection::default();
        selection.select_range(&["a", "b", "c"], "c", Some("a"), false);
        assert_eq!(selection.len(), 3);
        selection.activate("b");
        assert_eq!(selection.len(), 0);
        selection.select_range(&["c", "b", "a"], "a", None, false);
        assert!(selection.contains("a") && selection.contains("b"));
        assert!(!selection.contains("c"));
    }

    #[test]
    fn ranges_follow_current_order_and_keep_the_anchor() {
        let mut selection = AssetListSelection::default();
        selection.select_range(&["a", "b", "c", "d"], "d", Some("b"), false);
        assert_eq!(selection.len(), 3);
        selection.select_range(&["a", "b", "c", "d"], "a", None, false);
        assert_eq!(selection.len(), 2);
        assert!(selection.contains("a") && selection.contains("b"));
        selection.select_range(&["a", "b", "c", "d"], "d", None, true);
        assert_eq!(selection.len(), 4);
    }

    #[test]
    fn removed_items_and_anchor_are_pruned() {
        let mut selection = AssetListSelection::default();
        selection.activate("b");
        selection.select_range(&["a", "b"], "a", None, false);
        selection.retain(|hash| hash != "b");
        assert!(selection.contains("a"));
        assert_eq!(selection.len(), 1);
        selection.select_range(&["a", "c"], "c", None, false);
        assert_eq!(selection.hashes().cloned().collect::<Vec<_>>(), ["c"]);
        selection.clear();
        assert_eq!(selection.len(), 0);
    }
}
