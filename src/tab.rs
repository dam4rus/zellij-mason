use std::cmp::Ordering;

use zellij_tile::prelude::*;

use crate::Rect;

#[derive(Debug, Default, Clone, Copy)]
pub struct TabState {
    selected: Option<usize>,
}

impl TabState {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn offset_selected(&mut self, offset: isize) {
        self.selected = Some(
            self.selected
                .map(|selected_index| match offset.cmp(&0) {
                    Ordering::Greater => selected_index.saturating_add(offset.unsigned_abs()),
                    Ordering::Less => selected_index.saturating_sub(offset.unsigned_abs()),
                    Ordering::Equal => selected_index,
                })
                .unwrap_or(0),
        );
    }

    pub fn select_next(&mut self) {
        self.selected = Some(self.selected.map(|i| i.saturating_add(1)).unwrap_or(0));
    }

    pub fn select_prev(&mut self) {
        self.selected = Some(self.selected.map(|i| i.saturating_sub(1)).unwrap_or(0));
    }
}

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
