# Changelog

All notable changes to this project will be documented in this file.
Format: [keepachangelog.com](https://keepachangelog.com) · Versioning: [semver.org](https://semver.org)

## [Unreleased]

## [0.4.0] - 2026-06-30

### Added
- Expanded unit test coverage from 5 to 23 tests. Includes tests for `ratio_color`, monthly/daily/weekly reset intervals, executable search fallback, and configuration serialization.
- A thread-local path override helper `TEST_CONFIG_PATH` for clean, isolated configuration loading/saving unit tests without altering global host environment variables.

### Changed
- Modal Budget Editor: Real-time visual input validation. Fields highlight in red (`COLOR_DANGER`) and show a `⚠ Valid number required` helper if user inputs empty or non-numeric values.
- Navigation discoverability: Tabs are now numbered (`1 Overview`, `2 AI Agents`, etc.) to show keybind hotkeys, and edit keys dynamically hide for uninstalled agents.
- Sessions Tab: Replaced confusing mock data entries with a clear and friendly "No recent sessions found" empty state component.

### Fixed
- Backward compatibility: Added `#[serde(default)]` to the `model_limits` config field to prevent older configuration schemas from failing deserialization and being destructively overwritten by defaults.
- Keybind mapping hints: Corrected settings help menus to mention `Enter / +/-` for cycling values instead of `←→ / Enter` (which previously changed global tabs).
- Version checking: Fixed logic in `AgentScanner::get_version` where failure status codes from `--version` executions prevented running the fallback `-v` version check.

## [0.3.0] - 2026-06-25

### Changed
- Complete TUI visual layout refinement for a significantly improved terminal experience.
- Replaced static `●/○` pulse indicator in header with an animated braille spinner (`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`) and live agent count display.
- Active tab now renders with solid background highlight (text on primary color) instead of foreground-only highlight.
- Tab bar uses `│` dividers for cleaner visual separation.
- Progress bars upgraded to gradient-style blocks (`█▓░`) for a smoother fill appearance.
- Overview stat boxes now show contextual icons (`⬡`, `⬢`, `◈`) and values are vertically centered.
- Quota gauge cards use `title_bottom` to display agent tier without consuming inner space.
- Agent sidebar entries are visually separated with spacing; uninstalled agents render with `DIM` style.
- "Not installed" agent panel replaced verbose text with structured centered lines and icons.
- Agent detail hint bar replaced with inline keybind pills (`s`, `↑↓` with colored backgrounds).
- Sessions table status column uses solid-background badge (` ✔ OK `) and session hashes highlighted in sky blue.
- Quotas tab now has primary-colored border on the main table for visual hierarchy.
- Settings tab main card uses primary-colored border; config path highlighted in `COLOR_INFO`.
- Guide/Info panels use inline keybind pills instead of plain text shortcuts.
- Budget modal re-centered using a pure `centered_rect()` helper, gains a dark background (`Rgb(18,20,28)`), shadow effect, `title_bottom` with controls, and shows the current limit before input.
- Footer keybinds replaced with colored pill-style spans for all tabs.

### Added
- New `COLOR_INFO` (sky blue `Rgb(80,184,255)`) for informational text (hashes, paths, tier labels).
- New `COLOR_DIM` for secondary/background chrome separating active content from UI structure.
- New `spinner_frame()` helper for braille spinner animation.
- New `ratio_color()` helper to centralize soft/hard threshold color logic.
- New `centered_rect()` helper for modal positioning.
- New `kpill()` helper to render keybind pill spans consistently across footer and hint bars.
- Symbolic constants `SYM_ARROW`, `SYM_BLOCK_FULL`, `SYM_BLOCK_HALF`, `SYM_BLOCK_EMPTY`, `SYM_SEP` for consistent UI glyphs.

## [0.2.1] - 2026-06-25

### Fixed
- Updated README.md with crates.io installation guide, corrected Agy database telemetry paths, and aligned config schema.

## [0.2.0] - 2026-06-25

### Added
- New unit test module (`src/tests.rs`) covering Base64 decoding, JWT parsing, and configuration defaults.
- Type alias `ScanResult` for better code readability in background scanning threads.
- Proportional dynamic model limits mapping for Codex, OpenCode, Agy, and Zed based on user tiers and configuration.
- Support for `custom` quota flag in configuration to track manually overridden limits from the TUI.

### Changed
- Refactored `agent.rs` to use idiomatic Rust patterns (`saturating_sub`, `is_multiple_of`, `vec!`).
- Updated architecture documentation (`docs/wiki/architecture.md`) with current design decisions and ADRs.
- Improved development and hygiene guides in the wiki to reflect strict quality mandates.
- Removed Gemini-CLI support completely from the application (removed logic, configuration parameters, and UI tables).

### Fixed
- Resolved 23 code quality issues identified by Clippy (linting).
- Fixed potential logic errors in Base64 padding and saturating arithmetic.
- Cleaned up syntax garbage and formatting issues in `ui.rs` and `main.rs`.

## [0.1.0] - 2026-06-04

### Added

- Updated TUI themes to use `Color::Reset` for backgrounds, preserving the terminal's native background colors and transparency.
- Added support for `QuotaType::Monthly` to track billing and token quotas on a monthly frequency.
- Implemented dynamic monthly reset calculation targeting the 1st of the next calendar month.
- Integrated live token and cost telemetry inside Codex and OpenCode by querying their respective SQLite databases.
- Made OpenCode use Monthly quota tracking globally, dynamically querying the last active provider (e.g., OpenAI vs GitHub Copilot) from its SQLite database to apply cost and subscription rules.
- Updated Overview tab stats boxes to display Total Tokens Used and Cumulative spend across all active assistants.
- Enhanced AI Agents details panel to dynamically present token consumption and cost details.
- Standardized default Pro model limits for Gemini to 50 requests/day matching actual Google AI Studio free tier limits.
- Applied FMG Repository Development Standard.
- Initial project structure with TUI for monitoring AI agent quotas.
