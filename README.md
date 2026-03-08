# Simple Todo TUI (Rust)

A keyboard-first todo app for the terminal, built with `ratatui` + `crossterm`.

## Run

```bash
cargo run
```

## Keybindings

- `j`: move down
- `k`: move up
- `a`: add todo
- `e`: edit selected todo
- `x`: toggle complete
- `d`: delete selected todo (with confirm)
- `f`: cycle filter (`All` -> `Active` -> `Completed`)
- `.`: toggle help menu
- `Enter`: confirm in modal (add/edit/delete)
- `Esc`: cancel current modal
- `q`: quit

## Data file location

By default, todos are stored in:

- macOS: `~/Library/Application Support/simple-todo-tui/todos.json`
- Linux: `$XDG_DATA_HOME/simple-todo-tui/todos.json` (or fallback under `~/.local/share/...`)
- Fallback (if no platform data dir): `./todos.json`
