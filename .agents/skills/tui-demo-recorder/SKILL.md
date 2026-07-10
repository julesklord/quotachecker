---
name: tui-demo-recorder
description: Record animated demo GIFs for TUI and CLI applications using VHS.
---

# TUI & CLI Demo Recorder

This skill enables Antigravity to quickly set up, configure, and record animated `.gif` demos for Terminal User Interface (TUI) or Command Line Interface (CLI) applications using **VHS** (by Charmbracelet) and **ttyd**.

## When to use

Use this skill when:
- The user requests a usage demo, recording, or GIF showcasing a terminal application.
- The project is a CLI tool or interactive TUI application.
- You need to automate interactions and create a clean, reproducible terminal capture.

## Dependencies

Before recording, ensure the required utilities are installed on the host system:
1. **VHS**: The compiler for tape files.
2. **ttyd**: Shared terminal emulator over the web (required by VHS).
3. **Chromium / Google Chrome**: Headless browser used by VHS to capture screenshots of the terminal frames.
4. **ffmpeg**: Used to compile the captured screenshots into the final GIF.

### Installing Dependencies

- **Arch Linux / CachyOS**:
  ```bash
  sudo pacman -S --noconfirm ttyd vhs chromium ffmpeg
  ```
- **macOS** (Homebrew):
  ```bash
  brew install charmbracelet/tap/vhs ttyd ffmpeg
  ```
- **Debian / Ubuntu**:
  ```bash
  # Install go first if needed, then:
  go install github.com/charmbracelet/vhs@latest
  # Ensure ttyd, chromium-browser/chrome, and ffmpeg are in $PATH
  ```

---

## Recording Workflow

### 1. Compile the target binary
For performance and lag-free recordings, compile the CLI or TUI app in release mode (e.g. `cargo build --release` or equivalent).

### 2. Identify Monospace Fonts
Check what monospace fonts are installed on the system using:
```bash
fc-list : family | sort -u | grep -iE 'mono|nfm|nerd'
```
Look for compact or compressed fonts like `ZedMono NFM`, `Terminess Nerd Font Mono`, `RobotoMono NFM`, or `ProggyClean`.

### 3. Create the Tape Script (`docs/demo.tape`)
Write a `.tape` script defining the terminal parameters and input sequences.

#### Reference Template:
```tape
# Output path for the animated GIF
Output docs/demo.gif

# Visual Settings
Set FontSize 12
Set FontFamily "ZedMono NFM"   # Compact monospace font
Set Width 1000
Set Height 600
Set Theme "TokyoNight"         # Vibrant, dark theme (TokyoNight, Dracula, Catppuccin Mocha)
Set Padding 10

# Command to launch the application (hidden from the viewer)
Hide
Type "./target/release/your-app-binary"
Enter
Sleep 1.5s
Show

# --- Interaction Sequence ---
# 1. Show the main screen/overview
Sleep 4.5s

# 2. Navigate tabs or switch views
Tab
Sleep 1.5s

# 3. List navigation (Down, Up keys)
Down
Sleep 0.8s
Down
Sleep 0.8s
Up
Sleep 1s

# 4. Modify settings (type characters or trigger key combinations)
Type "+"
Sleep 1s
Type "-"
Sleep 1s

# 5. Open and edit modal (Example)
Type "s"
Sleep 1.5s
Type "120"
Sleep 1s
Enter
Sleep 3s

# 6. Clean Exit
Type "q"
Sleep 1s
```

### 4. Compile the Tape
Run the compiler from the root of the project:
```bash
vhs docs/demo.tape
```

### 5. Add to README
Update the `README.md` file (typically under the header or overview section) to display the demo:
```markdown
## Demo

![Demo](docs/demo.gif)
```
