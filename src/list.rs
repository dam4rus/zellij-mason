use std::{
    collections::{BTreeMap, BTreeSet},
    iter::once,
};

use zellij_tile::ui_components::{NestedListItem, print_nested_list_with_coordinates};

use crate::Rect;

#[derive(Debug, Default, Clone)]
pub struct ListState {
    selected_tree_index: Option<usize>,
    expansions: BTreeSet<usize>,
    item_index_tree: ItemIndexTree,
    scroll: usize,
}

impl ListState {
    pub fn selected_tree_index(&self) -> Option<usize> {
        self.selected_tree_index
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected_tree_index().and_then(|selected| {
            self.item_index_tree
                .flatten()
                .into_iter()
                .position(|i| i == selected)
        })
    }

    pub fn select_next(&mut self) {
        self.selected_tree_index = Some(match self.selected_tree_index {
            Some(selected) => self
                .item_index_tree
                .next(selected)
                .unwrap_or(selected.saturating_add(1)),
            None => 0,
        })
    }

    pub fn select_prev(&mut self) {
        self.selected_tree_index = Some(
            self.selected_tree_index
                .and_then(|i| self.item_index_tree.prev(i))
                .unwrap_or(0),
        );
    }

    pub fn selected_path(&self) -> Option<Vec<usize>> {
        self.selected_tree_index.and_then(|selected| {
            let mut path = Vec::new();
            self.item_index_tree.path(selected, &mut path).then(|| {
                path.reverse();
                path
            })
        })
    }

    pub fn scroll_selected_to_view(&mut self, rect: Rect) {
        if let Some(selected_index) = self.selected() {
            if selected_index < self.scroll {
                self.scroll = selected_index;
            } else if selected_index > self.scroll + (rect.height - 1) {
                self.scroll = selected_index - (rect.height - 1);
            }
        }
    }

    pub fn expand(&mut self, index: usize) {
        self.expansions.insert(index);
    }

    pub fn collapse(&mut self, index: usize) {
        self.expansions.remove(&index);
    }
}

#[derive(Debug, Default, Clone)]
pub struct ListItem {
    item: NestedListItem,
    children: Vec<ListItem>,
    indent: usize,
}

impl ListItem {
    pub fn new(item: NestedListItem) -> Self {
        Self {
            item,
            children: Vec::new(),
            indent: 0,
        }
    }

    pub fn with_children(mut self, children: impl IntoIterator<Item = ListItem>) -> Self {
        self.children = children
            .into_iter()
            .map(|child| child.with_indent(self.indent + 1))
            .collect();
        self
    }

    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }
}

pub fn render(list_items: Vec<ListItem>, rect: Rect, state: &mut ListState) {
    state.item_index_tree = list_items
        .iter()
        .flat_map(|list_item| once(list_item).chain(&list_item.children))
        .enumerate()
        .fold(ItemIndexTree::default(), |mut acc, (i, list_item)| {
            if list_item.indent == 0 {
                acc.0.insert(i, None);
            } else {
                let mut last_entry = acc.0.last_entry().unwrap();
                if state.expansions.contains(last_entry.key()) {
                    last_entry
                        .get_mut()
                        .get_or_insert_default()
                        .0
                        .insert(i, None);
                }
            }
            acc
        });

    if state.item_index_tree.0.is_empty() {
        state.selected_tree_index = None;
    } else {
        // Getting the last element should never fail when the tree is empty
        let last_index = state.item_index_tree.last().unwrap();
        state.selected_tree_index = Some(
            state
                .selected_tree_index
                .map(|selected_index| match selected_index {
                    index if index > last_index => last_index,
                    index => index,
                })
                .unwrap_or(0),
        );
    }

    state.scroll_selected_to_view(rect);

    let items = list_items
        .into_iter()
        .flat_map(|list_item| {
            once(list_item.item.indent(list_item.indent)).chain(
                list_item
                    .children
                    .into_iter()
                    .map(|child| child.item.indent(child.indent)),
            )
        })
        .enumerate()
        .filter(|(i, _)| state.item_index_tree.contains(*i))
        .skip(state.scroll)
        .take(rect.height)
        .map(|(i, list_item)| match state.selected_tree_index {
            Some(selected) if i == selected => list_item.selected(),
            _ => list_item,
        })
        .collect();

    print_nested_list_with_coordinates(items, rect.x, rect.y, Some(rect.width), Some(rect.height));
}

#[derive(Debug, Default, Clone)]
struct ItemIndexTree(BTreeMap<usize, Option<ItemIndexTree>>);

impl ItemIndexTree {
    fn flatten(&self) -> Vec<usize> {
        self.0
            .iter()
            .flat_map(|(i, children)| {
                once(*i).chain(children.iter().flat_map(|child| child.flatten()))
            })
            .collect()
    }

    fn contains(&self, index: usize) -> bool {
        self.0.iter().any(|(key, value)| {
            *key == index
                || value
                    .as_ref()
                    .map(|children| children.contains(index))
                    .unwrap_or(false)
        })
    }

    fn next(&self, index: usize) -> Option<usize> {
        self.0.iter().find_map(|(key, value)| {
            (*key > index)
                .then_some(*key)
                .or(value.as_ref().and_then(|children| children.next(index)))
        })
    }

    fn prev(&self, index: usize) -> Option<usize> {
        self.0.iter().rev().find_map(|(key, value)| {
            value
                .as_ref()
                .and_then(|children| children.prev(index))
                .or((*key < index).then_some(*key))
        })
    }

    fn last(&self) -> Option<usize> {
        self.0.last_key_value().map(|(key, value)| {
            value
                .as_ref()
                .and_then(|children| children.last())
                .unwrap_or(*key)
        })
    }

    fn path(&self, index: usize, out: &mut Vec<usize>) -> bool {
        match self.0.keys().position(|i| *i == index) {
            Some(position) => {
                out.push(position);
                true
            }
            None => match self.0.values().enumerate().find_map(|(i, children)| {
                children
                    .as_ref()
                    .map(|children| children.path(index, out))
                    .unwrap_or(false)
                    .then_some(i)
            }) {
                Some(position) => {
                    out.push(position);
                    true
                }
                None => false,
            },
        }
    }
}
