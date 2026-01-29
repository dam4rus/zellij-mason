use std::collections::BTreeMap;

use zellij_mason::{
    Rect,
    tab::{self, TabState},
};
use zellij_tile::prelude::*;

#[derive(Default)]
struct TabExample {
    state: TabState,
}

impl ZellijPlugin for TabExample {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
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
                    bare_key: BareKey::Tab,
                    key_modifiers,
                } => {
                    if key_modifiers.contains(&KeyModifier::Shift) {
                        self.state.select_prev();
                    } else {
                        self.state.select_next();
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        tab::render(
            &["My issues", "Sprint"],
            Rect {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            },
            &mut self.state,
        )
    }
}

register_plugin!(TabExample);
