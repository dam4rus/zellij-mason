//! A multi-level, nested list of text elements.
//!
//! Uses the builder pattern to construct a nested list.
//!
//! ```rust,no_run
//!  use zellij_mason::{
//!      Rect,
//!      list::{self, ListItem, ListState},
//!  };
//!  use zellij_tile::prelude::*;
//!
//!  let mut list_state = ListState::default();
//!  list::render(
//!      &[ListItem::new(NestedListItem::new("1st level"))
//!          .with_children([ListItem::new(NestedListItem::new("2nd level"))])],
//!      Rect {
//!          x: 0,
//!          y: 0,
//!          width: 100,
//!          height: 2,
//!      },
//!      &mut list_state,
//!  );
//! ```
use std::{
    collections::{BTreeMap, BTreeSet},
    iter::once,
};

use zellij_tile::ui_components::{NestedListItem, print_nested_list_with_coordinates};

use crate::Rect;

/// State of a nested list.
///
/// Can be constructed using the [ListState::default] constructor to make it compatible with Zellij's plugin system.
#[derive(Debug, Default, Clone)]
pub struct ListState {
    selected_tree_index: Option<usize>,
    expansions: BTreeSet<usize>,
    item_index_tree: ItemIndexTree,
    scroll: usize,
}

impl ListState {
    /// Select the next visible element in the list.
    pub fn select_next(&mut self) {
        self.selected_tree_index = Some(match self.selected_tree_index {
            Some(selected) => self
                .item_index_tree
                .next(selected)
                .unwrap_or(selected.saturating_add(1)),
            None => 0,
        })
    }

    /// Select the previous visible element in the list.
    pub fn select_prev(&mut self) {
        self.selected_tree_index = Some(
            self.selected_tree_index
                .and_then(|i| self.item_index_tree.prev(i))
                .unwrap_or(0),
        );
    }

    /// The path in the tree to the selected index.
    ///
    /// E.g. If the selected item is at the 1st indent level, a single element vector is returned.
    /// If the selected item is at the 2nd indent level, a two element vector is returned
    /// containing the index of the parent and the selected item.
    pub fn selected_path(&self) -> Option<Vec<ItemIndex>> {
        self.selected_tree_index
            .and_then(|selected| self.path(selected))
    }

    /// Expands the selected item, making it's children visible.
    pub fn expand_selected(&mut self) -> bool {
        match self.selected_tree_index {
            Some(selected_index) => {
                self.expansions.insert(selected_index);
                true
            }
            None => false,
        }
    }

    /// Collapses the selected item, making it's children invisible.
    pub fn collapse_selected(&mut self) -> bool {
        match self.selected_tree_index {
            Some(selected_index) => self.collapse(selected_index),
            _ => false,
        }
    }

    /// Maps the selected tree index to the visible list index.
    fn selected_list_index(&self) -> Option<usize> {
        self.selected_tree_index.and_then(|selected| {
            self.item_index_tree
                .flatten()
                .into_iter()
                .position(|i| i == selected)
        })
    }

    fn path(&self, index: usize) -> Option<Vec<ItemIndex>> {
        let mut path = Vec::new();
        self.item_index_tree.path(index, &mut path).then(|| {
            path.reverse();
            path
        })
    }

    fn scroll_selected_to_view(&mut self, rect: Rect) {
        if let Some(selected_index) = self.selected_list_index() {
            if selected_index < self.scroll {
                self.scroll = selected_index;
            } else if selected_index > self.scroll + (rect.height - 1) {
                self.scroll = selected_index - (rect.height - 1);
            }
        }
    }

    fn collapse(&mut self, index: usize) -> bool {
        if self.expansions.remove(&index) {
            if let Some(children) = self.item_index_tree.children(index) {
                for list_index in children.flatten() {
                    self.expansions.remove(&list_index);
                }
            }
            true
        } else {
            let Some(&[.., parent, _]) = self.path(index).as_deref() else {
                return false;
            };

            if self.expansions.remove(&parent.list_index) {
                self.selected_tree_index = Some(parent.list_index);
                true
            } else {
                false
            }
        }
    }
}

/// Describes an item in the list.
///
/// Can be constructed from a [NestedListItem].
#[derive(Debug, Default, Clone)]
pub struct ListItem {
    item: NestedListItem,
    children: Vec<ListItem>,
    indent: usize,
}

impl ListItem {
    /// Create a new [ListItem] from a [NestedListItem]
    pub fn new(item: NestedListItem) -> Self {
        Self {
            item,
            children: Vec::new(),
            indent: 0,
        }
    }

    /// Add children to the [ListItem].
    pub fn with_children(mut self, children: impl IntoIterator<Item = ListItem>) -> Self {
        self.children = children
            .into_iter()
            .map(|child| child.with_indent(self.indent + 1))
            .collect();
        self
    }

    fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self.children = self
            .children
            .into_iter()
            .map(|child| child.with_indent(indent + 1))
            .collect();
        self
    }

    /// Flattens the [ListItem] and all of it's children recursively.
    ///
    /// Meaning the returned container contains the [ListItem] itself as well.
    fn flatten(&self) -> Vec<&ListItem> {
        once(self)
            .chain(
                self.children
                    .iter()
                    .flat_map(|list_item| list_item.flatten()),
            )
            .collect()
    }
}

/// The index of an item in the [ListItem]'s element tree.
#[derive(Debug, Default, Clone, Copy)]
pub struct ItemIndex {
    /// The index in the list's tree.
    pub list_index: usize,
    /// The index of the element in it's indentation level, considered from it's parent.
    ///
    /// E.g. if the element is the first child of the second element at the root, it will be 0
    pub index_at_indent: usize,
}

/// Render list items.
///
/// * `state`: State of the list. Will be mutated based on the passed in `list_items` to update it's internals and
/// correct invalid selection indexes.
pub fn render(list_items: &[ListItem], rect: Rect, state: &mut ListState) {
    state.item_index_tree = list_items_to_index_tree(&list_items, state, &mut 0);

    if state.item_index_tree.0.is_empty() {
        state.selected_tree_index = None;
    } else {
        // Getting the last element should never fail when the tree is not empty
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
        .iter()
        .flat_map(|list_item| list_item.flatten())
        .map(|list_item| list_item.item.clone().indent(list_item.indent))
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

fn list_items_to_index_tree(
    list_items: &[ListItem],
    state: &ListState,
    current_index: &mut usize,
) -> ItemIndexTree {
    let mut index_tree = ItemIndexTree(BTreeMap::new());
    for list_item in list_items {
        if state.expansions.contains(current_index) && list_item.children.len() > 0 {
            index_tree.0.insert(
                *current_index,
                Some({
                    *current_index += 1;
                    list_items_to_index_tree(&list_item.children, state, current_index)
                }),
            );
        } else {
            index_tree.0.insert(*current_index, None);
            *current_index += list_item.flatten().len();
        }
    }
    index_tree
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ItemIndexTree(BTreeMap<usize, Option<ItemIndexTree>>);

impl ItemIndexTree {
    fn flatten(&self) -> Vec<usize> {
        self.0
            .iter()
            .flat_map(|(i, children)| {
                once(*i).chain(children.iter().flat_map(|children| children.flatten()))
            })
            .collect()
    }

    fn contains(&self, index: usize) -> bool {
        self.0.iter().any(|(list_index, children)| {
            *list_index == index
                || children
                    .as_ref()
                    .map(|children| children.contains(index))
                    .unwrap_or(false)
        })
    }

    fn next(&self, index: usize) -> Option<usize> {
        self.0.iter().find_map(|(list_index, children)| {
            (*list_index > index)
                .then_some(*list_index)
                .or(children.as_ref().and_then(|children| children.next(index)))
        })
    }

    fn prev(&self, index: usize) -> Option<usize> {
        self.0.iter().rev().find_map(|(list_index, children)| {
            children
                .as_ref()
                .and_then(|children| children.prev(index))
                .or((*list_index < index).then_some(*list_index))
        })
    }

    fn last(&self) -> Option<usize> {
        self.0.last_key_value().map(|(list_index, children)| {
            children
                .as_ref()
                .and_then(|children| children.last())
                .unwrap_or(*list_index)
        })
    }

    fn children(&self, index: usize) -> Option<&Self> {
        self.0.iter().find_map(|(list_index, children)| {
            if *list_index == index {
                children.as_ref()
            } else {
                children
                    .as_ref()
                    .and_then(|children| children.children(index))
            }
        })
    }

    fn path(&self, index: usize, out: &mut Vec<ItemIndex>) -> bool {
        match self.0.keys().enumerate().find_map(|(i, list_index)| {
            (*list_index == index).then_some(ItemIndex {
                list_index: *list_index,
                index_at_indent: i,
            })
        }) {
            Some(item_index) => {
                out.push(item_index);
                true
            }
            None => match self
                .0
                .iter()
                .enumerate()
                .find_map(|(i, (list_index, children))| {
                    children
                        .as_ref()
                        .map(|children| children.path(index, out))
                        .unwrap_or(false)
                        .then_some(ItemIndex {
                            list_index: *list_index,
                            index_at_indent: i,
                        })
                }) {
                Some(list_index) => {
                    out.push(list_index);
                    true
                }
                None => false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use zellij_tile::shim::NestedListItem;

    use crate::list::{ItemIndexTree, ListItem, ListState, list_items_to_index_tree};

    #[test]
    fn test_list_items_to_index_tree() {
        assert_eq!(
            ItemIndexTree(BTreeMap::from_iter([
                (0, Some(ItemIndexTree(BTreeMap::from_iter([(1, None)])))),
                (
                    3,
                    Some(ItemIndexTree(BTreeMap::from_iter([(
                        4,
                        Some(ItemIndexTree(BTreeMap::from_iter([(5, None)])))
                    )])))
                ),
                (6, None),
                (9, None)
            ])),
            list_items_to_index_tree(
                &[
                    ListItem::new(NestedListItem::new("foo"))
                        .with_children([ListItem::new(NestedListItem::new("bar"))
                            .with_children([ListItem::new(NestedListItem::new("baz"))])]),
                    ListItem::new(NestedListItem::new("foobar"))
                        .with_children([ListItem::new(NestedListItem::new("barfoo"))
                            .with_children([ListItem::new(NestedListItem::new("foobaz"))])]),
                    ListItem::new(NestedListItem::new("foo"))
                        .with_children([ListItem::new(NestedListItem::new("bar"))
                            .with_children([ListItem::new(NestedListItem::new("baz"))])]),
                    ListItem::new(NestedListItem::new("apple"))
                ],
                &ListState {
                    expansions: BTreeSet::from_iter([0, 3, 4]),
                    ..ListState::default()
                },
                &mut 0,
            )
        )
    }
}
