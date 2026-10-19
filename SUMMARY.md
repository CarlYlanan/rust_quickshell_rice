# owo-bar - Sekaipop Aesthetic Bar for Niri (Latest Update)

## What Was Updated

Successfully updated the owo-bar project to:
1. **Separate the bar into distinct pills** for each module (workspace, clock, tray) with spacing between them.
2. **Apply a dark blue gradient background** (miku blue version) for better text visibility.
3. **Reduce bar height to 42px**.
4. **Bold and center the clock module**.
5. **Revise the workspace module** to show:
   - Left: Label "WORKSPACE{Number}" where Number is the focused workspace.
   - Right: A tightly packed sequence of kanji buttons for each occupied workspace (left to right), hugged by the pill border.
6. **Maintain the Sekaipop aesthetic** with the three-layer pill system (shadow → navy outline → pale ring → white fill) applied to each pill.
7. **Keep the rice Niri backend** for accurate workspace tracking via niri msg --json event-stream.
8. **Retain kanji workspace indicators** (一, 二, 三, 四, 五, 六, 七, 八, 九, 〇) with per-workspace attribute colors.

## Key Changes

### Visual Design (theme.css)
- **Bar background**: Dark blue gradient (`#0a1522` to `#0f1a2d`)
- **Each pill**: 
  - Background: Slightly lighter dark blue gradient (`#0f1a2d` to `#1a263d`)
  - Three-layer system:
    - Shadow: `rgba(0,0,0,0.5)` (3px down)
    - Outline: `#1a1f2e` (navy, 2px inset)
    - Ring: `#a0c4ff` (light blue, 3px inset)
    - Fill: `rgba(255,255,255,0.05)` (subtle white for depth)
- **Spacing**: 2px padding and 2px margin between pills for separation
- **Text colors**: 
  - Workspace label: JetBrains Mono bold, light gray (`#e0e0e0`)
  - Workspace buttons: Kanji colors (cyan/pink/yellow/lavender/mint) with navy text on active/focused
  - Clock: Light gray (`#e0e0e0`) for contrast, bold weight 900
  - Tray: Default icons with attention highlight (pink background)

### Technical Implementation (main.rs)
- **Bar height**: Changed constant `BAR_HEIGHT` from 44 to 42.
- **Module wrapping**: Each module is placed inside a vertical `gtk::Box` with CSS class "pill".
  - Workspace pill contains a label (for "WORKSPACE{Number}") and a horizontal box (for kanji sequence buttons).
  - Clock pill contains the clock button.
  - Tray pill contains the tray box.
- **Layout**: 
  - Bar window has dark blue gradient background (via CSS).
  - Inner container (`bar-modules`) is transparent horizontal box with 8px spacing.
  - Pills expand to fill available space while maintaining internal alignment (start/center/end).
- **Workspace logic**:
  - Label updates to show focused workspace ID.
  - Buttons box is cleared and repopulated with buttons for each occupied workspace (kanji only).
  - Each button is clickable to focus that workspace via niri command.
- **Clock**: Updates every second with `%m月%d日 %H:%M` format, bold font.
- **System tray**: Unchanged status-notifier integration.

## Files Updated
```
0w0/
├── theme.css          - Dark blue gradient, pill styling, adjusted dimensions
├── src/main.rs        - Bar height 42px, separated pills, workspace label + kanji sequence, bold clock
├── README.md          - Updated documentation
└── SUMMARY.md         - This file
```

## Current Status
✅ **Compiles successfully** (`cargo build` works)
✅ **All tests pass** (workspace logic verified)
✅ **Core bar functional** with separated pills, dark blue gradient, bold centered clock, and revised workspace module
🔲 **Runtime display** requires Niri + Wayland environment to visualize
⚠️ **Minor warnings** about unused fields (expected during development)

## How to Run
```bash
# Build
cargo build --release

# Run (requires Niri + Wayland)
cargo run
```

## Design Notes
- The bar now consists of three distinct pills: [Workspace] [Clock] [Tray]
- Each pill has rounded corners, subtle shadow, and the signature sekaipop three-layer effect
- Dark blue gradient background ensures text/icons are visible while maintaining the playful, bright aesthetic
- Workspace module shows a label indicating the focused workspace and a sequence of kanji for occupied workspaces
- Clock remains centered and bold with date/time format
- System tray remains right-aligned with pinned/unpinned logic and attention hints

## Next Steps (Future Work)
1. Add CSS transitions for workspace scaling animations
2. Implement configurable bar position/settings
3. Create popup systems (calendar, media, battery)
4. Add additional modules (resources, etc.)
5. Polish and optimize for release

The owo-bar now successfully combines the separated pill layout with a dark blue gradient background, bold centered clock, and revised workspace module while preserving the Sekaipop visual language and rice's accurate Niri workspace integration.