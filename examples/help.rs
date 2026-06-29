use zellij_mason::{
    Rect,
    help::{self, KeyBinding},
};
use zellij_tile::prelude::*;

#[derive(Default)]
struct HelpExample;

impl ZellijPlugin for HelpExample {
    fn render(&mut self, rows: usize, cols: usize) {
        help::render_keybindings(
            &[
                KeyBinding {
                    key: String::from("<Ctrl-r>"),
                    summary: String::from("Refresh"),
                },
                KeyBinding {
                    key: String::from("<Ctrl-c>"),
                    summary: String::from("Quit"),
                },
            ],
            Rect {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            },
        );
        help::render_keybindings(
            &[KeyBinding {
                key: String::from("<Ctrl-c>"),
                summary: String::from("Quit"),
            }],
            Rect {
                x: 0,
                y: 1,
                width: cols,
                height: rows,
            },
        )
    }
}

register_plugin!(HelpExample);
