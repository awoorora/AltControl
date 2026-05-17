# AltCtrl - Keyboard-Driven Mouse Navigation Overlay

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![GPL-3.0 LICENSE](https://img.shields.io/badge/gpl-3.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)

A lightweight, always-on-top overlay that lets you navigate your screen using keyboard shortcuts. Press `Alt+Ctrl` to activate a 10×30 grid overlay, then type two keys to move your mouse to any cell. Perfect for accessibility, productivity workflows, or just having fun with your keyboard.

## Quick Start
PowerShell - `Start-Process -WindowStyle Hidden -FilePath "[your file path]\altctrl.exe"`

## Showcase

https://github.com/user-attachments/assets/df3d828d-65bc-4836-a0a8-6f732241237b

https://youtu.be/dDK0VC_prtA

## Features

- **Global Hotkey Activation**: Press `Alt+Left Ctrl` anywhere to open the overlay
- **Visual Grid Overlay**: See a color-coded 10×30 grid mapped to your keyboard
- **Two-Key Navigation**: Type a column key (`A,S,D,F,G,H,J,K,L,;`) then a row key (`Q,W,E,...,/,`) to move your mouse
- **No Key Leakage**: Keys typed while the overlay is open are captured and never sent to background apps
- **Dynamic DPI Support**: Automatically adapts to your display's scale factor and resolution
- **Smooth Rendering**: Anti-aliased text and crisp grid lines rendered with softbuffer
- **Click Simulation**: Map `Left Ctrl` / `Alt` to left/right mouse clicks

## How to Use

1. **Launch** the app - it runs silently in the background
2. **Press `Alt+Left Ctrl`** - the overlay appears with a grid
3. Type **two keys**:
   - **First**: A column key from the middle keyboard row: `A S D F G H J K L ;`
   - **Second**: Any row key from the full QWERTY layout: `Q W E R T Y U I O P A S ... N M , . /`
1. Watch your mouse **snap** to the center of the selected cell
2. Overlay closes **automatically** - return to your work

### Example Sequences
| Keys      | Result                                |
| --------- | ------------------------------------- |
| `A` → `Q` | Mouse moves to top-left cell (AQ)     |
| `D` → `W` | Mouse moves to upper-middle cell (DW) |
| `;` → `/` | Mouse moves to bottom-right cell (;/) |
| `ESC`     | Close overlay immediately             |

### Grid Layout
**Columns(10)** : A S D F G H J K L
**Rows(30)** : QWERTY row → indices 0-9, ASDF row → indices 10-19, ZXCV row → indices 20-29
Each cell is 1/10th of screen width × 1/30th of screen height. Mouse lands at cell center.

## Installation

### Prerequisites
- Rust 1.70 or later ([install](https://www.rust-lang.org/tools/install))
- Windows 10/11, Linux (X11), or macOS

### Build from Source
```bash
git clone https://github.com/awoorora/AltControl
cd AltControl
cargo build --release
```

The executable will be at `target/release/altctrl.exe` (Windows) or `target/release/altctrl` (Linux/macOS).

### Dependencies

All dependencies are managed via `Cargo.toml`:
```toml
[dependencies]
winit = "0.30"      # Cross-platform windowing
rdev = "0.14"       # Global keyboard/mouse hooks
softbuffer = "0.4"  # CPU-side pixel rendering
rusttype = "0.9"    # Font rasterization
```

## Configuration

No config file needed, AltCtrl auto-detects your display settings. Advanced users can modify constants in `src/overlay.rs`:
```rust
// Font size (scales with resolution)
font_px: 25.0,

// Column colors (0xRRGGBB format, 20% opacity)
let column_colors = [0xf86565, 0xffc766, 0xe6ff66, 0x68ff66, 0x66f7ff];

// Grid dimensions (auto-detected, but hardcoded fallback)
grid_width: 1920,
grid_height: 1080,
```

## Privacy & Security

- **No telemetry**: AltCtrl does not collect or transmit any data
- **Local execution only**: All keyboard/mouse simulation happens on your machine
- **Open source**: Audit the code yourself — no hidden behavior
- **Minimal permissions**: Only requires standard input simulation APIs (no admin/root needed on most systems)

> **Note**: On Windows, global keyboard hooks may trigger antivirus warnings. AltCtrl is safe — the source is public and builds reproducibly.

## Troubleshooting

|Issue|Solution|
|---|---|
|Overlay doesn't appear on second `Alt+Ctrl`|Ensure you're running the latest build; try `cargo clean && cargo build --release`|
|Keys still appear in background apps|The overlay must gain keyboard focus — if it doesn't, try clicking the overlay once|
|Mouse moves to wrong location|Check your display scaling (AltCtrl auto-detects, but custom DPI settings may interfere)|
|Font looks blurry|Try a different `.ttf` file in `assets/`; adjust `font_px` in `overlay.rs`|
|App won't start on Linux/Wayland|AltCtrl currently supports X11; use XWayland or switch to X11 session|

## Contributing

Contributions welcome! Please:

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/amazing-idea`)
3. Commit your changes (`git commit -m 'Add amazing idea'`)
4. Push to the branch (`git push origin feat/amazing-idea`)
5. Open a Pull Request

## License

GPL-3.0 License — see [LICENSE](https://www.gnu.org/licenses/gpl-3.0.en.html) for details.

## Acknowledgments

- [winit](https://github.com/rust-windowing/winit) for cross-platform windowing
- [rdev](https://github.com/Narsil/rdev) for global input simulation
- [softbuffer](https://github.com/rust-windowing/softbuffer) for simple CPU rendering
- The Rust community for making systems programming accessible
