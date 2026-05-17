<div align="center">
<img width="1920" height="580" alt="altctrl banner" src="https://github.com/user-attachments/assets/12fa5188-b852-4ccd-ba8e-0217a815c5ed" />
<h1>AltCtrl - Keyboard-Driven Cursor Navigation</h1>
</div>

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A lightweight, always-on-top overlay that lets you navigate your entire screen without touching the mouse. Press `Alt+Left Ctrl` to summon a labeled 10×30 grid, then type up to three keys to place the cursor with increasing precision - then optionally click, all from the keyboard.

## Quick Start

PowerShell: `Start-Process -WindowStyle Hidden -FilePath "[your file path]\altctrl.exe"`

## Showcase

https://github.com/user-attachments/assets/df3d828d-65bc-4836-a0a8-6f732241237b

https://youtu.be/dDK0VC_prtA

## Features

- **Global Hotkey**: Press `Alt+Left Ctrl` anywhere to open the overlay
- **Visual Grid**: Color-coded 10×30 grid with every cell labeled by its key combo
- **Progressive Precision**: Column → cell → left/right third, with the mouse moving at each step so you always see where you're headed
- **Click Simulation**: `Left Ctrl` for left click, `Alt` for right click - usable at any point in the sequence
- **No Key Leakage**: All keypresses while the overlay is open are captured and never reach background apps
- **Auto DPI Detection**: Adapts to your display's resolution and scale factor automatically
- **Crisp Rendering**: Anti-aliased labels and grid lines via softbuffer

## How to Use

1. **Launch** the app - it runs silently in the background
2. **Press `Alt+Left Ctrl`** - the overlay appears over your entire screen
3. **Press a column key** from the home row: `A S D F G H J K L ;`
   - The mouse moves to the **center of that column**
4. **Press a row key** from anywhere on the keyboard: `Q W E R T Y U I O P` / `A S D F G H J K L ;` / `Z X C V B N M , . /`
   - The mouse moves to the **center of that cell**
5. **Optionally refine** to the left or right third of the cell:
   - Any **left-side key** (`Q W E R T`, `A S D F G`, `Z X C V B`) → left third
   - Any **right-side key** (`Y U I O P`, `H J K L ;`, `N M , . /`) → right third
6. At any point, **press `Left Ctrl`** to left-click or **`Alt`** to right-click at the current position
7. Or press **`ESC`** to close without clicking

The overlay closes automatically after a click.

### Example Sequences

| Keys | Result |
|---|---|
| `A` | Mouse moves to center of column A |
| `A` → `Q` | Mouse moves to cell AQ |
| `A` → `Q` → left-side key | Mouse moves to left third of AQ |
| `A` → `Q` → right-side key | Mouse moves to right third of AQ |
| `A` → `Q` → `Ctrl` | Left-click at center of AQ |
| `A` → `Q` → left-side key → `Ctrl` | Left-click at left third of AQ |
| `A` → `A` → `A` | Left third of cell AA (same key works in every slot) |
| `;` → `/` → `Alt` | Right-click at cell ;/ |
| `ESC` | Close overlay, mouse stays put |

### Grid Layout

**Columns (10):** `A S D F G H J K L ;` - left to right across the screen  
**Rows (30):** QWERTY row (0–9) → ASDF row (10–19) → ZXCV row (20–29)  
Each cell is 1/10th of screen width × 1/30th of screen height.

## Installation

### Prerequisites

- Rust 1.70 or later ([install](https://www.rust-lang.org/tools/install))
- Windows 10/11, or Linux (X11)

### Build from Source

```bash
git clone https://github.com/awoorora/AltControl
cd AltControl
cargo build --release
```

The binary will be at `target/release/altctrl.exe` (Windows) or `target/release/altctrl` (Linux).

### Dependencies

```toml
[dependencies]
winit = "0.30"      # Cross-platform windowing
rdev = "0.14"       # Global keyboard/mouse hooks
softbuffer = "0.4"  # CPU-side pixel rendering
rusttype = "0.9"    # Font rasterization
```

## Configuration

No config file needed - display resolution is detected automatically at runtime. To tweak rendering, edit these values near the top of `src/overlay.rs`:

```rust
font_px: 25.0,  // label size

let column_colors = [0xf86565, 0xffc766, 0xe6ff66, 0x68ff66, 0x66f7ff];  // 0xRRGGBB, 20% opacity
```

## Privacy & Security

- **No telemetry** - nothing is collected or transmitted
- **Local only** - all input simulation happens on your machine
- **Open source** - full source is here; no hidden behavior
- **No elevated privileges** - standard input APIs only, no admin/root required on most systems

> **Note:** On Windows, global keyboard hooks may trigger antivirus warnings. AltCtrl is safe - the source is public and builds are reproducible.

## Troubleshooting

| Issue | Solution |
|---|---|
| Keys still reach background apps | The overlay must have keyboard focus; click it once if needed |
| Mouse moves to the wrong spot | Check your display scaling settings; custom DPI may interfere |
| Font looks blurry | Swap in a different `.ttf` in `assets/`; adjust `font_px` in `overlay.rs` |
| Click lands on the wrong target | Try increasing the sleep delay in `perform_click_and_close` (default: 80ms) |
| Doesn't start on Linux | X11 only - use XWayland or switch to an X11 session if on Wayland |

## Contributing

Contributions welcome!

1. Fork the repo
2. Create a feature branch: `git checkout -b feat/your-idea`
3. Commit your changes: `git commit -m 'Add your idea'`
4. Push: `git push origin feat/your-idea`
5. Open a Pull Request

## License

[GNU General Public License v3.0](https://www.gnu.org/licenses/gpl-3.0) - see [LICENSE](./LICENSE) for the full text.

## Acknowledgments

- [winit](https://github.com/rust-windowing/winit) - cross-platform windowing
- [rdev](https://github.com/Narsil/rdev) - global input hooks and simulation
- [softbuffer](https://github.com/rust-windowing/softbuffer) - simple CPU-side rendering
- The Rust community for making systems programming approachable
