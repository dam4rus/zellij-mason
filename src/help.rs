//! Render a list of keybindings with summary to give hints to the user
//!
//! ```rust,no_run
//! use zellij_mason::{
//!     Rect,
//!     help::{self, KeyBinding},
//! };
//!
//! help::render_keybindings(
//!     &[KeyBinding {
//!         key: String::from("<Ctrl-c>"),
//!         summary: String::from("Quit"),
//!     }],
//!     Rect {
//!         x: 0,
//!         y: 0,
//!         width: 100,
//!         height: 1,
//!     },
//! );
//! ```
use zellij_tile::prelude::*;

use crate::Rect;

/// Define a key binding with details about it's usage.
#[derive(Debug, Default, Clone)]
pub struct KeyBinding {
    /// The key of the binding.
    pub key: String,
    /// Short summary of the key binding.
    pub summary: String,
}

/// Render a list of keybindings in a single line.
pub fn render_keybindings(key_bindings: &[KeyBinding], rect: Rect) {
    match &key_bindings {
        [key_binding] => {
            render_command_help(key_binding, rect.x, rect.y);
            ()
        }
        [head, tail @ ..] => {
            let (x, y) = render_command_help(head, rect.x, rect.y);
            tail.into_iter().fold((x, y), |(mut x, mut y), elem| {
                (x, y) = render_help_separator(x, y);
                (x, y) = render_command_help(elem, x, y);
                (x, y)
            });
        }
        &[] => (),
    }
}

fn render_command_help(key_binding: &KeyBinding, mut x: usize, y: usize) -> (usize, usize) {
    print_text_with_coordinates(
        Text::new(&key_binding.key).color_range(3, 0..),
        x,
        y,
        None,
        None,
    );

    x += key_binding.key.chars().count();
    let separator = " - ";
    print_text_with_coordinates(Text::new(separator), x, y, None, None);

    x += separator.chars().count();

    print_text_with_coordinates(Text::new(&key_binding.summary), x, y, None, None);
    x += key_binding.summary.chars().count();
    (x, y)
}

fn render_help_separator(x: usize, y: usize) -> (usize, usize) {
    let text = ", ";
    print_text_with_coordinates(Text::new(text), x, y, None, None);

    (x + text.chars().count(), y)
}
