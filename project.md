cleansh/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── logger.rs
│   ├── commands/
│   │   └── cleansh.rs
│   ├── tools/
│   │   └── sanitize_shell.rs
│   └── ui/                     # NEW: All UI-related logic
│       ├── mod.rs              # Exports UI components
│       └── theme.rs            # Defines colors, styles, table formatting
│       └── output_format.rs    # Functions for printing structured data (tables, lists, diffs)
├── config/
│   └── default_rules.yaml
├── .env
├── .gitignore
├── Cargo.toml
├── README.md
├── LICENSE (MIT)