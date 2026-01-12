# zellij-mason
Layout and rendering helpers for built-in zellij plugin widgets

## Quickstart

Add `zellij-mason` and the appropriate version of `zellij-tile` to your dependencies.

```bash
cargo add zellij-tile@0.43.1 zellij-mason
```

Then use one of the provided components in conjunction with the widgets provided by `zellij-tile`.

```rust
use zellij_mason::{
    Rect,
    table::{self, TableState},
};
use zellij_tile::prelude::*;

#[derive(Default)]
pub struct MyPlugin;

impl ZellijPlugin for MyPlugin {
    fn render(&mut self, rows: usize, cols: usize) {
        let mut table_state = TableState::default();
        table::render(
            ["ID", "Name"],
            &[[Text::new("0"), Text::new("Robert")]],
            Rect {
                x: 0,
                y: 0,
                width: cols,
                height: rows,
            },
            &mut table_state,
        );
    }
}
```

Check the documentation of each module for more details.

## Contribution

There is currently no contribution guide but feel free to open issues and PRs.

