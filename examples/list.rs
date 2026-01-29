use zellij_mason::{
    Rect,
    list::{self, ListItem, ListState},
};
use zellij_tile::prelude::*;

#[derive(Default)]
struct ListExample {
    state: ListState,
}

impl ZellijPlugin for ListExample {
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
                KeyWithModifier {
                    bare_key: BareKey::Right,
                    ..
                } => self.state.expand_selected(),
                KeyWithModifier {
                    bare_key: BareKey::Left,
                    ..
                } => self.state.collapse_selected(),
                _ => false,
            },
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        list::render(
            &[
                ListItem::new(NestedListItem::new("1st level - 0"))
                    .with_children([ListItem::new(NestedListItem::new("2nd level"))
                        .with_children([ListItem::new(NestedListItem::new("3rd level"))])]),
                ListItem::new(NestedListItem::new("1st level - 1")),
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

register_plugin!(ListExample);
