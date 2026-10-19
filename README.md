# owo-bar

Sekaipop-style top bar for Niri workspace compositor with kanji workspace indicators.

## Features

- Top bar layout with workspace module (left), clock (center), system tray (right)
- **Separated pills**: Each module is in its own pill with spacing between them
- Sekaipop aesthetic: three-layer pill system (shadow → navy outline → pale ring → white fill)
- **Dark blue gradient background** (miku blue version) for better text visibility
- **Bar height**: 42px
- Kanji workspace indicators (一, 二, 三, 四, 五, 六, 七, 八, 九, 〇) with per-workspace colors
- **Workspace label**: Shows "WORKSPACE{Number}" where Number is the focused workspace
- **Kanji sequence**: To the right of the label, a sequence of kanji for occupied workspaces (left to right, tightly hugged by pill border)
- Active workspace button scales up with smooth animation
- Niri backend for accurate workspace state tracking
- System tray with status-notifier support (pinned/unpinned apps, attention hints)
- **Bold centered clock** with date format (月日 時間:分)

## Architecture

- **GTK 4** with **layer-shell** for Wayland/Niri integration
- **Rust** implementation (port of rice/sekaipop concepts)
- **CSS-driven styling** for sekaipop visual fidelity
- **Modular design**: separate components for workspaces, clock, tray

## Components

### Workspace Module (`workspace_label` + `workspace_group`)
- **Left**: Label showing "WORKSPACE {focused_id}" (e.g., "WORKSPACE 3")
- **Right**: Horizontal sequence of kanji buttons for each occupied workspace
- Buttons have no spacing, tightly packed so the pill hugs the sequence
- Active workspace button: scales to 118% with color background
- Occupied workspace: shows workspace color
- Per-workspace attribute colors: cyan, pink, yellow, lavender, mint (rotating)
- Smooth animations using OutBack easing curves

### Clock Module (`clock_button`)
- Centered display
- Format: 月日 時間:分 (e.g., 10月19日 14:30)
- JetBrainsMono font, **bold** (weight 900)
- Updates every second

### System Tray Module (`tray_group`)
- Right-aligned
- Pinned apps capsule (placeholder)
- Unpinned items in dropdown (via status-notifier)
- Attention highlighting for apps needing user input
- Hover effects

## Design Specifications (from sekaipop)

### Bar Dimensions
- Height: 42px
- Corner radius: 12px
- Padding: 0 10px (window margins)

### Three-Layer Pill System (per pill)
1. **Hard shadow**: 3px down, rgba(0,0,0,0.5)
2. **Navy outline**: 2px solid inset, #1a1f2e  
3. **Pale ring**: 3px inset, #a0c4ff (light blue)
4. **White fill**: rgba(255,255,255,0.05) (subtle)

### Pill Gradient
- Background: linear-gradient(#0f1a2d, #1a263d)

### Bar Background Gradient
- linear-gradient(#0a1522, #0f1a2d)

### Workspace Badges
- Size: 28x28px
- Border radius: 4px
- Font: Fira Sans, 16px, ExtraBold
- Active text color: #1a1f2e (navy) on workspace color background
- Idle/occupied text color: workspace color

### Colors
```css
--navy: #1a1f2e;
--shadow: rgba(0,0,0,0.5);
--pale-ring: #a0c4ff;
--white-fill: rgba(255,255,255,0.05);
--ws-cyan: #00d0d0;
--ws-pink: #ff4d6d;
--ws-yellow: #ffe066;
--ws-lavender: #c0b0ff;
--ws-mint: #a0ffc8;
--accent-cyan: #00bebe;
--accent-mint: #66ffcc;
--bar-gradient: #0a1522 → #0f1a2d;
--pill-gradient: #0f1a2d → #1a263d;
```

## Implementation Status

✅ **Core bar structure** - GTK layer-shell window with three zones (left/center/right) as separate pills
✅ **Workspace module** - Label + kanji sequence with per-workspace colors and active scaling
✅ **Clock module** - Centered bold clock with date formatting
✅ **Tray module** - Status-notifier integration for system tray items
✅ **Styling** - Sekaipop-inspired CSS with three-layer pill system and dark blue gradient
✅ **Niri backend** - Workspace state parsing from niri msg --json event-stream
✅ **Build system** - Cargo project with proper dependencies
✅ **Tests** - Unit tests for workspace kanji mapping, color logic, and serialization

🔲 **Animations** - CSS transitions for workspace scaling (currently using transform transition)
🔲 **Full popup system** - Calendar, media, battery popups (phase 2)
🔲 **Config options** - Bar position, size, visibility settings
🔲 **Additional modules** - Battery, resources, media, etc.

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

Requires:
- Niri workspace compositor
- GTK 4 development libraries
- gtk4-layer-shell
- status-notifier daemon (makoctl)
- WiFi/Bluetooth tools (optional for full functionality)

## Notes

This is a faithful recreation of the sekaipop bar aesthetic adapted for:
- Niri instead of Hyprland compositor
- Rust/GTK4 instead of QML/Quickshell
- Separated pills with dark blue gradient background
- Kanji workspace indicators with label and sequence
- Layer-shell protocol for Wayland bar positioning

The bar follows sekaipop's visual language precisely while maintaining rice's Niri integration for accurate workspace tracking.