# appgrep

**appgrep** is a unified CLI tool that discovers, lists, and provides information about all installed applications on a Linux system. It aggregates apps from desktop entry files, Flatpak, Snap, and standalone/AppImage installs into a single queryable interface with structured output formats designed for composability.

## Installation

```bash
cargo install --git https://github.com/kikinit/appgrep
```

Or build from source:

```bash
git clone https://github.com/kikinit/appgrep
cd appgrep
cargo build --release
# Binary at target/release/appgrep
```

## Usage

```
appgrep [OPTIONS] <COMMAND>

Commands:
  list              List all discovered applications
  info <name>       Show detailed info about an application
  search <query>    Fuzzy search for applications
  has <name>        Check if installed (exit 0=yes, 1=no)
  run <name>        Launch an application
  path <name>       Print exec command for an application

Options:
  -f, --format <FORMAT>    table|json|tsv|names|exec  [default: table]
  -s, --source <SOURCE>    desktop|flatpak|snap|standalone (repeatable)
      --no-color           Disable colored output
  -h, --help
  -V, --version
```

### Examples

**List all applications:**

```bash
appgrep list
```

```
┌──────────────┬──────────────────────┬──────────┬─────────────────────────┐
│ Name         │ Exec                 │ Source   │ Description             │
├──────────────┼──────────────────────┼──────────┼─────────────────────────┤
│ Firefox      │ /usr/bin/firefox     │ desktop  │ Web Browser             │
│ LocalSend    │ flatpak run org...   │ flatpak  │ Share files locally     │
│ Spotify      │ snap run spotify     │ snap     │ Music for everyone      │
│ Godot        │ ~/Applications/...   │ standalone│                        │
└──────────────┴──────────────────────┴──────────┴─────────────────────────┘
```

**Search for an application:**

```bash
appgrep search firefox
```

**Check if an application is installed:**

```bash
appgrep has firefox && echo "Firefox is installed"
```

**Get the exec path for scripting:**

```bash
appgrep path firefox
# /usr/bin/firefox
```

**Show detailed info:**

```bash
appgrep info firefox
```

```
Name:        Firefox
Exec:        /usr/bin/firefox
Source:      desktop
Location:    /usr/share/applications/firefox.desktop
Icon:        firefox
Categories:  Network, WebBrowser
Description: Web Browser
```

**JSON output:**

```bash
appgrep --format json list
```

**Filter by source:**

```bash
appgrep --source flatpak list
appgrep --source desktop --source snap list
```

**Launch an application:**

```bash
appgrep run firefox
```

## Composability

appgrep is designed to pipe into other tools:

```bash
# Interactive app picker with fzf
appgrep --format names list | fzf | xargs appgrep run

# Query JSON with jq
appgrep --format json list | jq '.[] | select(.source == "flatpak") | .name'

# App launcher with rofi
appgrep --format names list | rofi -dmenu | xargs appgrep run

# List all exec paths
appgrep --format exec list

# Check in scripts
if appgrep has firefox; then
    appgrep run firefox
fi

# TSV for spreadsheet-friendly output
appgrep --format tsv list > apps.tsv
```

## Output Formats

| Format  | Flag             | Description                          |
|---------|------------------|--------------------------------------|
| table   | `--format table` | Colored table (default)              |
| json    | `--format json`  | JSON array/object                    |
| tsv     | `--format tsv`   | Tab-separated with header            |
| names   | `--format names` | One name per line (for piping)       |
| exec    | `--format exec`  | One exec command per line            |

## Application Sources

| Source     | How it discovers                                     |
|------------|------------------------------------------------------|
| desktop    | Scans XDG `.desktop` files in standard directories   |
| flatpak    | Runs `flatpak list --app`                            |
| snap       | Runs `snap list` + reads snap `.desktop` metadata    |
| standalone | Scans `~/Applications`, `~/.local/bin`, `/opt`, etc. |

## License

MIT
