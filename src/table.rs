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

#[derive(Debug, Default, Clone)]
pub struct Options {
    // Truncates text at the given column index to make sure it's at least partially visible.
    // Useful when a column's text can be pretty long and the columns after the given column is not as important.
    pub truncate_text_at_column: Option<usize>,
}

pub fn render<const N: usize>(
    header: [impl ToString; N],
    rows: &[[Text; N]],
    rect: Rect,
    state: &mut TableState,
) {
    render_with_options(header, rows, rect, state, None);
}

pub fn render_with_options<const N: usize>(
    header: [impl ToString; N],
    rows: &[[Text; N]],
    rect: Rect,
    state: &mut TableState,
    options: Option<Options>,
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

    let visible_rows = rows
        .iter()
        .enumerate()
        .skip(state.scroll)
        .take(rect.height - 1)
        .collect::<Vec<_>>();

    let column_max_width = options
        .and_then(|option| option.truncate_text_at_column)
        .map(|truncate_text_at_column| {
            (
                truncate_text_at_column,
                calculate_max_column_width(
                    visible_rows.iter().map(|(_, row)| *row),
                    rect,
                    truncate_text_at_column,
                ),
            )
        });

    let table = visible_rows.into_iter().fold(
        Table::new().add_row(
            header
                .map(|header_column| header_column.to_string())
                .to_vec(),
        ),
        |acc, (y, row)| {
            let row = row
                .iter()
                .enumerate()
                .map(|(x, column)| {
                    let column = match column_max_width {
                        Some((truncate_text_at_column, max_width))
                            if truncate_text_at_column == x && column.len() > max_width =>
                        {
                            Text::new(format!("{}᳟", &column.content()[..max_width]))
                        }
                        _ => column.clone(),
                    };
                    match state.selected {
                        Some(selected_index) if selected_index == y => column.selected(),
                        _ => column,
                    }
                })
                .collect();

            acc.add_styled_row(row)
        },
    );

    print_table_with_coordinates(table, rect.x, rect.y, Some(rect.width), Some(rect.height));
}

fn calculate_max_column_width<'a, const N: usize>(
    rows: impl Iterator<Item = &'a [Text; N]>,
    rect: Rect,
    column: usize,
) -> usize {
    rows.fold(rect.width, |acc, row| {
        acc.min(
            row.iter()
                .take(column)
                .fold(rect.width.saturating_sub(column + 1), |acc, column| {
                    acc.saturating_sub(column.len())
                }),
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::{Rect, table::calculate_max_column_width};
    use zellij_tile::shim::Text;

    #[test]
    fn test_calculate_max_column_width() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 0,
        };
        let expected_width = rect.width - (" ".len()) - ("Summary".len()) - 3;
        assert_eq!(
            calculate_max_column_width(
                [&[
                    Text::new(" "),
                    Text::new("Summary"),
                    Text::new("Long description for a given row"),
                ]]
                .into_iter(),
                rect,
                2,
            ),
            expected_width
        );
    }
}
