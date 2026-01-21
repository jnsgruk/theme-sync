# theme-sync

<a href="https://snapcraft.io/theme-sync"><img alt="theme-sync" src="https://snapcraft.io/theme-sync/badge.svg"/></a>

Synchronize theme preferences across multiple terminal tools based on your GNOME appearance settings.

While many tools now support automatic theme switching based on system settings, many do not. This
utility aims to bridge that gap by monitoring GNOME's theme state, then substituting theme names
in configuration files.

This was a quick project to scratch my own itch!

## Features

- Watch `gsettings` for GNOME color-scheme changes (`monitor` command)
- Set light/dark theme one-off (`set` command), optionally overriding the detected preference
- Configurable via YAML (loaded with [`confy`](https://docs.rs/confy))
- Supports per-app token replacements and optional reload commands

## Installation

The easiest way to use `theme-sync` is using the snap:

```bash
# Enable the user-daemons experimental feature in snapd
snap set system experimental.user-daemons=true

# Install the snap
sudo snap install theme-sync

# Enable the app to edit files in $HOME/.config
sudo snap connect theme-sync:dot-config

# (Optional) Enable the app to send signals to other apps to reload config
sudo snap connect theme-sync:process-control
```

Once the snap is functional, you can also opt to start it in the background as a service:

```bash
snap start --enable --user theme-sync.daemon
```

You can build and install `theme-sync` using `cargo`:

```bash
cargo install --git https://github.com/jnsgruk/theme-sync
```

Or run with Nix:

```bash
nix run github:jnsgruk/theme-sync#theme-sync
```

## Usage

```bash
# Apply the current system theme once
theme-sync set

# Apply a specific theme regardless of GNOME settings
theme-sync set --theme dark

# Monitor GNOME theme changes continuously
theme-sync monitor

# Use a custom configuration file
theme-sync --config /path/to/config.yaml monitor
```

## Configuration

Configurations are stored in YAML and loaded by `confy`. By default, the file lives at
`~/.config/theme-sync/config.yaml`. If the config file does not exist, it will be created
automatically.

Example configuration:

```yaml
apps:
  - name: zellij
    path: .config/zellij/config.kdl
    light_token: catppuccin-latte
    dark_token: catppuccin-macchiato

  - name: helix
    path: .config/helix/config.toml
    light_token: catppuccin_latte
    dark_token: catppuccin_macchiato
    reload_cmd: pkill -USR1 hx

  - name: bottom
    path: .config/bottom/bottom.toml
    light_token: '"default-light"'
    dark_token: '"default"'
```

Each `app` entry defines:

- `name`: friendly label for logging
- `path`: file path (relative to `$HOME`) to update
- `light_token` / `dark_token`: strings swapped for light/dark modes
- `reload_cmd`: optional command run via `bash -c` after updating the file

## Logging

The application uses `env_logger` with a default log level of `info`. Override by setting
`RUST_LOG`, e.g.:

```bash
RUST_LOG=debug theme-sync monitor
RUST_LOG=info theme-sync monitor
RUST_LOG=warn theme-sync monitor
```

## License

This project is licensed under the Apache 2.0 License.
