use std::cmp::Ordering;
use zellij_tile::prelude::*;

use crate::Rect;

#[derive(Debug, Default, Clone, Copy)]
pub struct TableState {
    selected: Option<usize>,
    scroll: usize,
}

impl TableState {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
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

    pub fn scroll_selected_to_view(&mut self, rect: Rect) {
        if let Some(selected_index) = self.selected {
            if selected_index < self.scroll {
                self.scroll = selected_index;
            } else if selected_index > self.scroll + (rect.height - 2) {
                self.scroll = selected_index - (rect.height - 2);
            }
        }
    }
}

pub fn render<const N: usize>(
    header: [impl ToString; N],
    rows: &[[Text; N]],
    rect: Rect,
    state: &mut TableState,
) {
    if rows.is_empty() {
        state.selected = None;
    } else {
        state.selected = Some(
            state
                .selected
                .map(|selected_index| {
                    if selected_index >= rows.len() {
                        rows.len() - 1
                    } else {
                        selected_index
                    }
                })
                .unwrap_or(0),
        );
    }

    state.scroll_selected_to_view(rect);

    let table = rows
        .iter()
        .enumerate()
        .skip(state.scroll)
        .take(rect.height - 1)
        .fold(
            Table::new().add_row(
                header
                    .map(|header_column| header_column.to_string())
                    .to_vec(),
            ),
            |acc, (i, row)| {
                let row = row
                    .iter()
                    .map(|column| match state.selected {
                        Some(selected_index) if selected_index == i => column.clone().selected(),
                        _ => column.clone(),
                    })
                    .collect();

                acc.add_styled_row(row)
            },
        );

    print_table_with_coordinates(table, rect.x, rect.y, Some(rect.width), Some(rect.height));
}
