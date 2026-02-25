# appgrep

**appgrep** is a unified CLI tool that discovers, lists, and provides information about all installed applications on a Linux system. It aggregates apps from desktop entry files, Flatpak, Snap, standalone/AppImage installs, Cargo, npm, dpkg, rpm, pacman, and Homebrew into a single queryable interface with structured output formats designed for composability.

## Installation

### Pre-built binaries

Download from [GitHub Releases](https://github.com/kikinit/appgrep/releases):

| Target | Description |
|--------|-------------|
| `appgrep-x86_64-unknown-linux-gnu` | Standard Linux x86_64 |
| `appgrep-aarch64-unknown-linux-gnu` | Raspberry Pi, ARM servers |
| `appgrep-x86_64-unknown-linux-musl` | Static binary, Alpine/containers |
| `appgrep-aarch64-unknown-linux-musl` | Static ARM binary |

```bash
# Example: download and install
curl -L https://github.com/kikinit/appgrep/releases/latest/download/appgrep-x86_64-unknown-linux-gnu -o appgrep
chmod +x appgrep
sudo mv appgrep /usr/local/bin/
```

### From source

```bash
cargo install --git https://github.com/kikinit/appgrep
```

Or build manually:

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
  doctor            Show provider status and diagnostics
  completions       Generate shell completion script

Options:
  -f, --format <FORMAT>    table|json|tsv|names|exec  [default: table]
  -s, --source <SOURCE>    desktop|flatpak|snap|standalone|cargo|npm|dpkg|rpm|pacman|brew (repeatable)
      --no-color           Disable colored output
      --stats              Show source statistics after output
  -h, --help
  -V, --version
```

### Examples

**List all applications:**

```bash
appgrep list
```

```
+──────────────+──────────────────────+──────────+─────────────────────────+
| Name         | Exec                 | Source   | Description             |
+──────────────+──────────────────────+──────────+─────────────────────────+
| Firefox      | /usr/bin/firefox     | desktop  | Web Browser             |
| LocalSend    | flatpak run org...   | flatpak  | Share files locally     |
| Spotify      | snap run spotify     | snap     | Music for everyone      |
| Godot        | ~/Applications/...   | standalone|                        |
| ripgrep      | ~/.cargo/bin/rg      | cargo    |                         |
| curl         | /usr/bin/curl        | dpkg     | command line tool...    |
+──────────────+──────────────────────+──────────+─────────────────────────+
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
appgrep --source cargo list
```

**Launch an application:**

```bash
appgrep run firefox
```

**Show statistics:**

```bash
appgrep --stats list
# Stats: 142 desktop, 8 flatpak, 0 snap, 3 standalone, 12 cargo, 0 npm, 45 dpkg, 0 rpm, 0 pacman, 0 brew — total 210
```

**System diagnostics:**

```bash
appgrep doctor
```

```
appgrep doctor

Providers:
  ✓ desktop        142 apps   Firefox, GIMP, VLC
  ✓ flatpak          8 apps   LocalSend, Flatseal, Extension Manager
  ✗ snap           unavailable
  ✓ standalone       3 apps   Godot, Cura, Logseq
  ✓ cargo           12 apps   ripgrep, fd, bat
  ✗ npm            unavailable
  ✓ dpkg            45 apps   curl, git, ffmpeg
  ✗ rpm            unavailable
  ✗ pacman         unavailable
  ✗ brew           unavailable

Total: 210 apps (before dedup)
```

**Shell completions:**

```bash
# bash
eval "$(appgrep completions bash)"

# zsh
appgrep completions zsh > ~/.zfunc/_appgrep

# fish
appgrep completions fish | source
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

# Count apps per source
appgrep --format json list | jq -r '.[].source' | sort | uniq -c | sort -rn

# Find all CLI tools from dpkg
appgrep --source dpkg --format names list
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

| Source     | How it discovers                                                  |
|------------|-------------------------------------------------------------------|
| desktop    | Scans XDG `.desktop` files in standard directories                |
| flatpak    | Runs `flatpak list --app`                                         |
| snap       | Runs `snap list` + reads snap `.desktop` metadata                 |
| standalone | Scans `~/Applications`, `~/.local/bin`, `/opt`, etc.              |
| cargo      | Scans `~/.cargo/bin/` for Rust-installed tools                    |
| npm        | Scans global npm bin directory for Node.js tools                  |
| dpkg       | Lists Debian/Ubuntu packages with executables (no .desktop file)  |
| rpm        | Lists RPM packages with executables (Fedora/RHEL/CentOS)         |
| pacman     | Lists pacman packages with executables (Arch/Manjaro)             |
| brew       | Lists Homebrew formulae with executables (Linuxbrew)              |

## License

MIT
