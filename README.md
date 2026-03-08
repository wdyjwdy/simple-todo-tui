# Simple Todo TUI (Rust)

A keyboard-first todo app for the terminal, built with `ratatui` + `crossterm`.

## Run

```bash
cargo run
```

## Keybindings

- `Left`: focus group panel
- `Right`: focus todo panel
- `j`: move down in focused panel
- `k`: move up in focused panel
- `a`: add in focused panel (group or todo)
- `e`: edit in focused panel (group or todo)
- `d`: delete in focused panel (group or todo, with confirm)
- `x`: toggle selected todo complete (todo panel only)
- `f`: cycle todo filter (`All` -> `Active` -> `Completed`, todo panel only)
- `.`: toggle help panel
- `Enter`: confirm in modal (add/edit/delete)
- `Esc`: cancel current modal
- `q`: quit

## Data file location

By default, todos are stored in:

- macOS: `~/Library/Application Support/simple-todo-tui/todos.json`
- Linux: `$XDG_DATA_HOME/simple-todo-tui/todos.json` (or fallback under `~/.local/share/...`)
- Fallback (if no platform data dir): `./todos.json`
