# Gantt Builder

A simple Gantt chart app built with Rust and Dioxus. With the frontend in WASM.

## Getting started

Install the Dioxus CLI:

```
cargo binstall dioxus-cli
```

Run the dev server:

```
dx serve
```

Open http://localhost:8080.

## Features

- Add, edit, delete tasks
- Drag to move or resize task bars
- Import/export Excel (.xlsx) and CSV files
- Dependency arrows between tasks
- Double-click a task bar to edit dates, progress, and color

## Example data

There's a sample CSV in `examples/` you can import to try things out.
