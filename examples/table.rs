use zellij_mason::{
    Rect,
    table::{self, TableState},
};
use zellij_tile::prelude::*;

#[derive(Default)]
struct TableExample {
    state: TableState,
}

impl ZellijPlugin for TableExample {
    fn load(&mut self, _configuration: std::collections::BTreeMap<String, String>) {
        subscribe(&[EventType::Key]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) => match key {
                KeyWithModifier {
                    bare_key: BareKey::Esc,
                    ..
                } => {
                    close_self();
                    true
                }
                KeyWithModifier {
                    bare_key: BareKey::Down,
                    ..
                } => {
                    self.state.select_next();
                    true
                }
                KeyWithModifier {
                    bare_key: BareKey::Up,
                    ..
                } => {
                    self.state.select_prev();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        table::render(
            ["Summary", "Description"],
            &[
                [
                    Text::new("Do something"),
                    Text::new("Something should be done"),
                ],
                [
                    Text::new("Another task"),
                    Text::new("Just another boring task"),
                ],
            ],
            Rect {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            },
            &mut self.state,
        );
    }
}

register_plugin!(TableExample);
