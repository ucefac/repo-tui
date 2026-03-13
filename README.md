> **Statement**: This project was developed entirely via live coding.

[中文文档](./README.zh.md)

# repotui

A terminal user interface (TUI) tool for browsing and managing GitHub repositories.

## Features

- 🔍 **Real-time Search**: Instantly filter repositories as you type
- ⌨️ **Keyboard-driven**: Vim-style navigation (j/k, g/G, Ctrl+d/u)
- 🎯 **Quick Actions**: Open repositories in WebStorm, VS Code, or Finder
- 🔐 **Secure**: Command whitelisting, path validation, no shell injection
- 🚀 **Fast**: Async repository scanning, virtual list rendering
- 🎨 **Themed**: Dark and light theme support
- 📁 **Directory Discovery**: Shows all directories in the configured main directory, with git repositories displaying branch information and non-git directories showing "Not a git repo"

## Installation

```bash
# Clone the repository
git clone https://github.com/repotui/repotui.git
cd repotui

# Build in release mode
cargo build --release

# Run
./target/release/repotui
```

## Quick Start

On first launch, you'll be prompted to select your main repositories directory.

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` | Go to top |
| `G` | Go to bottom |
| `/` | Focus search |
| `Esc` | Clear search |
| `Enter` / `o` | Open action menu |
| `c` | cd + cloud (claude) |
| `w` | Open in WebStorm |
| `v` | Open in VS Code |
| `f` | Open in Finder |
| `r` | Refresh list |
| `?` | Show help |
| `q` | Quit |

## Configuration

Configuration file location: `~/.config/repotui/config.toml`

```toml
# Configuration version
version = "1.0"

# Main directory to scan for repositories
main_directory = "~/Developer/github"

# Editor configuration
[editors]
webstorm = "/Applications/WebStorm.app/Contents/MacOS/webstorm"
vscode = "code"

# UI configuration
[ui]
theme = "dark"
show_git_status = true
show_branch = true

# Security configuration
[security]
allow_symlinks = false
max_search_depth = 2
```

## Security

repotui implements several security measures:

- **Command Whitelisting**: Only pre-approved commands can be executed
- **Path Validation**: All paths are validated to be within the configured main directory
- **No Shell Injection**: Uses `Command::current_dir()` instead of shell `cd` commands
- **Configuration Permissions**: Config file is set to `chmod 600` (Unix only)

## Development

### Requirements

- Rust 1.75+
- Git (for repository status detection)

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# With git2 backend (optional)
cargo build --features git2
```

### Test

```bash
# Run all tests
cargo test

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Benchmark

```bash
cargo bench
```

### Lint

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## Architecture

repotui follows the **Elm architecture** pattern:

```
┌─────────────┐     ┌─────────────┐
│     Msg     │────▶│   Update    │
└─────────────┘     └──────┬──────┘
                           │
                           ▼
┌─────────────┐     ┌─────────────┐
│    View     │◀────│    Model    │
└─────────────┘     └─────────────┘
```

### Modules

- `app/`: Application state (Model, Msg, Update)
- `ui/`: UI rendering (View)
- `handler/`: Keyboard event handling
- `config/`: Configuration management
- `repo/`: Repository discovery and status
- `action/`: Command execution
- `runtime/`: Async task execution

## Roadmap

- [ ] Recent repositories history
- [ ] Repository favorites
- [ ] Fuzzy search
- [ ] File system watcher for auto-refresh
- [ ] Custom keybindings
- [ ] Multiple profiles

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- [ratatui](https://github.com/ratatui/ratatui) - TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [broot](https://github.com/Canop/broot) - File browser TUI inspiration
- [lazygit](https://github.com/jesseduffield/lazygit) - Git TUI inspiration
