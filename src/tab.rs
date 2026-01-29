//! List of ribbons used as a tab line.
//!
//! ```rust,no_run
//! use zellij_mason::{
//!     Rect,
//!     tab::{self, TabState},
//! };
//!
//! let mut tab_state = TabState::default();
//! tab::render(
//!     &["My issues", "Sprint"],
//!     Rect {
//!         x: 0,
//!         y: 0,
//!         width: 100,
//!         height: 1,
//!     },
//!     &mut tab_state,
//! );
//! ```
use zellij_tile::prelude::*;

use crate::Rect;

/// State of a tab line.
///
/// Can be constructed using the [TabState::default] constructor to make it compatible with Zellij's plugin system.
#[derive(Debug, Default, Clone, Copy)]
pub struct TabState {
    selected: Option<usize>,
}

impl TabState {
    /// The selected index.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Select the next tab.
    pub fn select_next(&mut self) {
        self.selected = Some(self.selected.map(|i| i.saturating_add(1)).unwrap_or(0));
    }

    /// Select the previous tab.
    pub fn select_prev(&mut self) {
        self.selected = Some(self.selected.map(|i| i.saturating_sub(1)).unwrap_or(0));
    }
}

/// Render a list of tabs in a tab line at the specified coordinates.
///
/// * `state`: State of the tabline. Will be mutated based on the number of tabs passed in
/// to make sure the selected index is always within the lists boundary.
pub fn render(names: &[impl ToString], rect: Rect, state: &mut TabState) {
    if names.is_empty() {
        state.selected = None;
        return;
    } else {
        state.selected = Some(
            state
                .selected
                .map(|selected_index| {
                    if selected_index >= names.len() {
                        names.len() - 1
                    } else {
                        selected_index
                    }
                })
                .unwrap_or(0),
        );
    }

    let mut x = rect.x;
    for (i, name) in names.iter().enumerate() {
        let name = name.to_string();
        let name_len = name.len();
        let text = match state.selected {
            Some(selected) if selected == i => Text::new(name).selected(),
            _ => Text::new(name),
        };
        print_ribbon_with_coordinates(text, x, rect.y, None, None);
        x += name_len + RIBBON_OVERFLOW;
    }
}

const RIBBON_OVERFLOW: usize = 4;
