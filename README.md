# QuotaChecker-TUI

[![Crates.io](https://img.shields.io/crates/v/quotachecker-tui.svg?style=flat-square)](https://crates.io/crates/quotachecker-tui)
[![Crates.io Downloads](https://img.shields.io/crates/d/quotachecker-tui.svg?style=flat-square)](https://crates.io/crates/quotachecker-tui)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)

A terminal dashboard to track usage limits and token consumption of your local AI coding assistants.

## What it does

Tracks requests and token usage in the background by querying the local database and log files of your installed coding assistants.

### Supported Assistants

| Assistant | Data Source | Collected Metrics | Reset Freq |
| :--- | :--- | :--- | :--- |
| Codex | `~/.codex/state_5.sqlite` | Sessions, requests, tokens | Daily |
| OpenCode | `~/.local/share/opencode/opencode.db` | Sessions, requests, tokens, spent cost | Monthly |
| Agy | `~/.gemini/antigravity-cli/log/` | CLI prompts, command logs | Weekly |
| Zed | `~/.local/share/zed/threads/threads.db` | Active threads | Daily |

## Installation

### From crates.io (Recommended)
```bash
cargo install quotachecker-tui
```

### From Git
```bash
cargo install --git https://github.com/julesklord/quotachecker-tui
```

### From Source
```bash
git clone https://github.com/julesklord/quotachecker-tui.git
cd quotachecker-tui
cargo build --release
```
The compiled binary is located at `./target/release/quotachecker-tui`.

## Usage

Run the dashboard:
```bash
quotachecker-tui
```

### Keybindings

| Key | Action |
| :--- | :--- |
| `Tab` / `←` `→` | Switch tabs |
| `↑` `↓` | Navigate lists |
| `s` | Edit active assistant request limits |
| `+` / `-` | Modify values in Settings |
| `Enter` | Confirm and save inputs |
| `Esc` | Cancel modal |
| `r` | Force-trigger a background telemetry scan |
| `q` | Quit |

### Available Tabs

1. **Overview** — Aggregate costs, tokens, and requests across all assistants.
2. **AI Agents** — Versions, configurations, and quota breakdown for the selected assistant.
3. **Sessions** — Past sessions and telemetry logs.
4. **Quotas** — Usage gauges with warning thresholds.
5. **Settings** — Refresh intervals and visual themes.

## Configuration

The config file is located at `~/.config/quotachecker-tui/config.json`.

```json
{
  "refresh_rate_ms": 2000,
  "soft_limit_percent": 80.0,
  "hard_limit_percent": 100.0,
  "theme": "Cyan",
  "codex_quota": {
    "limit": 200,
    "custom": false
  },
  "opencode_quota": {
    "limit": 1000,
    "custom": false
  },
  "agy_quota": {
    "limit": 500,
    "custom": false
  },
  "zed_quota": {
    "limit": 300,
    "custom": false
  },
  "model_limits": {
    "gpt-5": 50,
    "gpt-4.1": 100,
    "claude-4.7": 150
  }
}
```

### Configuration Fields
- `refresh_rate_ms`: Delay in milliseconds between background scans.
- `soft_limit_percent`: Percentage where gauges turn yellow to warn the user.
- `hard_limit_percent`: Percentage where gauges turn red indicating limit is reached.
- `custom`: When set to `true`, the application respects your configured limit. When `false`, it defaults to the detected tier quota.

## Architecture

- **Asynchronous Telemetry**: A background thread reads SQLite databases using a `500ms` busy timeout to avoid write locks on active AI tools.
- **In-Memory Cache**: Shared config uses `Arc<RwLock<AppConfig>>` to prevent constant disk I/O.
- **Panic Hook**: Restores terminal state if the application crashes unexpectedly.

## License

MIT License. See [LICENSE](LICENSE).