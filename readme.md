# nk
configuration management for humans

## setup

- `~/.nk.yml`
```
machine: laptop-william
sources:
  # TODO: remove /nk once we're fully migrated over?
  - ~/Projects/config/nk
  - ~/Projects/config-private/nk
plugins:
  - ~/Projects/nk-plugin-pacman
```

- `cargo run`
