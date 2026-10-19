# Quickshell Research Report: Left-Side Menu with Fluid Animations

## Executive Summary

This document details the implementation of a left-side animated menu for the niri/GTK bar integration using Quickshell. The menu features:
- Spring-based slide animation from left edge
- White flash "blink" on open
- IPC interface for external triggering (from Rust bar)
- Clean QML architecture

---

## 1. IpcHandler API (Core Communication)

### Import
```qml
import Quickshell.Io
```

### Structure
```qml
IpcHandler {
    target: "menu"  // Unique identifier for `qs ipc call menu ...`

    // Functions callable via `qs ipc call menu <function> [args...]`
    function toggle(): void { /* implementation */ }
    function show(): void { visible = true }
    function hide(): void { visible = false }

    // Signals observable via `qs ipc wait menu` / `qs ipc listen menu`
    signal opened();
    signal closed();
}
```

### CLI Usage
```bash
# Call functions
qs ipc call menu toggle
qs ipc call menu show
qs ipc call menu hide

# Listen for signals (blocks until signal)
qs ipc wait menu opened

# List all registered targets
qs ipc show
```

### Type Annotations Required
Functions MUST have explicit type annotations:
- Arguments: `string`, `int`, `bool`, `real`, `color`
- Return: `void`, `string`, `int`, `bool`, `real`, `color`

```qml
function setOpacity(opacity: real): void { panel.opacity = opacity }
function getState(): string { return visible ? "open" : "closed" }
```

---

## 2. SpringAnimation Parameters (Fluid Drop Animation)

### Configuration
```qml
Behavior on panel.x {
    SpringAnimation {
        spring: 2.2      // Stiffness - higher = snappier
        damping: 0.22    // Damping ratio - lower = more overshoot/bounce
        mass: 0.7        // Mass - affects momentum
    }
}
```

### Physics Explanation
| Parameter | Value | Effect |
|-----------|-------|--------|
| `spring` | 2.2 | Moderate stiffness - responsive but not jittery |
| `damping` | 0.22 | Underdamped - slight overshoot for "pop" feel |
| `mass` | 0.7 | Light - quick response |

### Implementation Pattern
```qml
Rectangle {
    id: panel
    x: root.visible ? 0 : -root.width  // Target position
    width: 320
    height: parent.height
    
    Behavior on x {
        SpringAnimation { spring: 2.2; damping: 0.22; mass: 0.7 }
    }
}

// Trigger via visibility toggle
function toggle() {
    root.visible = !root.visible
    // x automatically animates via Behavior binding
}
```

---

## 3. Blink Flash Effect (White Overlay)

### Structure
```qml
Rectangle {
    id: panel
    // ... panel content ...
    
    // White flash overlay (on top of content)
    Rectangle {
        id: blinkFlash
        anchors.fill: parent
        color: "#ffffff"
        opacity: 0
        z: 100  // Above all content
    }
}
```

### Animation Sequence
```qml
SequentialAnimation {
    id: blinkAnim
    // Quick white flash
    NumberAnimation {
        target: blinkFlash
        property: "opacity"
        to: 0.9
        duration: 30
        easing.type: Easing.OutQuad
    }
    // Fade out
    NumberAnimation {
        target: blinkFlash
        property: "opacity"
        to: 0
        duration: 120
        easing.type: Easing.OutQuad
    }
}

// Trigger on open
function toggle() {
    visible = !visible
    if (visible) blinkAnim.start()
}
```

### Timing Rationale
- **30ms flash**: Perceptible but not distracting
- **120ms fade**: Smooth dissipation
- **Total 150ms**: Faster than spring animation (~300-500ms), creates "pop" feel

---

## 4. PanelWindow Configuration (Left Sidebar)

### Essential Properties
```qml
PanelWindow {
    id: root
    anchors.left: parent.left
    anchors.top: parent.top
    anchors.bottom: parent.bottom
    implicitWidth: 320        // Use implicitWidth, not width (deprecated)
    visible: false
    aboveWindows: true        // Replaces Qt.WindowStaysOnTopHint flag
    color: "transparent"      // For backdrop effect
}
```

### Deprecated → Current Mapping
| Deprecated | Current |
|------------|---------|
| `width: 320` | `implicitWidth: 320` |
| `height: 40` | `implicitHeight: 40` |
| `flags: Qt.Tool \| Qt.FramelessWindowHint \| Qt.WindowStaysOnTopHint` | `aboveWindows: true` |

---

## 5. Backdrop Overlay (Dim Background)

```qml
Rectangle {
    id: backdrop
    anchors.fill: parent
    color: "#00000000"
    opacity: 0
    Behavior on opacity { NumberAnimation { duration: 180 } }
}

// Sync with panel visibility
// In toggle(): backdrop.opacity = root.visible ? 0.4 : 0
// Or use PropertyAnimation/Behavior on root.visible
```

---

## 6. Complete Menu Implementation

### File: `LeftMenu.qml`
```qml
import QtQuick
import QtQuick.Controls
import Quickshell
import Quickshell.Io

PanelWindow {
    id: root
    anchors.left: parent.left
    anchors.top: parent.top
    anchors.bottom: parent.bottom
    implicitWidth: 320
    visible: false
    aboveWindows: true
    color: "transparent"

    // Backdrop
    Rectangle {
        id: backdrop
        anchors.fill: parent
        color: "#00000000"
        opacity: 0
        Behavior on opacity { NumberAnimation { duration: 180 } }
    }

    // Sliding Panel
    Rectangle {
        id: panel
        x: root.visible ? 0 : -root.width
        width: root.width
        height: parent.height
        color: "#1a1a2e"
        border.color: "#ff6b9d"
        border.width: 2

        Behavior on x {
            SpringAnimation { spring: 2.2; damping: 0.22; mass: 0.7 }
        }

        // Blink Flash Overlay
        Rectangle {
            id: blinkFlash
            anchors.fill: parent
            color: "#ffffff"
            opacity: 0
            z: 100
        }

        // Menu Content
        Column {
            anchors.fill: parent
            spacing: 16
            padding: 24

            Text {
                text: "MENU"
                font.pixelSize: 28
                font.bold: true
                color: "#ff6b9d"
                horizontalAlignment: Text.AlignHCenter
            }

            Repeater {
                model: [
                    { title: "Workspaces", action: "workspaces" },
                    { title: "Applications", action: "apps" },
                    { title: "Settings", action: "settings" },
                    { title: "Power", action: "power" }
                ]
                delegate: Item {
                    width: parent.width
                    height: 48

                    Rectangle {
                        id: bg
                        anchors.fill: parent
                        radius: 8
                        color: mouseArea.hovered ? "#ff6b9d44" : "transparent"
                        border.color: mouseArea.hovered ? "#ff6b9d" : "transparent"
                        border.width: 1
                    }

                    Row {
                        spacing: 12
                        anchors.centerIn: parent
                        Text { text: "▸"; font.pixelSize: 20; color: "#ff6b9d" }
                        Text { text: model.title; font.pixelSize: 16; color: "#ffffff" }
                    }

                    MouseArea {
                        id: mouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: root.onItemClicked(model.action)
                    }
                }
            }
        }
    }

    // Blink Animation
    SequentialAnimation {
        id: blinkAnim
        NumberAnimation { target: panel.blinkFlash; property: "opacity"; to: 0.9; duration: 30; easing.type: Easing.OutQuad }
        NumberAnimation { target: panel.blinkFlash; property: "opacity"; to: 0; duration: 120; easing.type: Easing.OutQuad }
    }

    // IPC Handler
    IpcHandler {
        id: ipc
        target: "menu"

        function toggle(): void { root.toggle() }
        function show(): void { root.visible = true }
        function hide(): void { root.visible = false }
    }

    // Internal Functions
    function toggle() {
        visible = !visible
        backdrop.opacity = visible ? 0.4 : 0
        if (visible) blinkAnim.start()
    }

    function onItemClicked(action) {
        console.log("Menu action:", action)
        toggle()
        // Could emit IPC signal here if needed
    }
}
```

---

## 7. Rust Bar Integration

### Trigger Button (in GTK/Rust)
```rust
let workspace_trigger = gtk::Button::new();
let trigger_icon = gtk::Image::from_icon_name("view-grid-symbolic");
trigger_icon.set_pixel_size(20);
workspace_trigger.set_child(Some(&trigger_icon));
workspace_trigger.set_tooltip_text(Some("Open Menu"));
workspace_trigger.connect_clicked(move |_| {
    let _ = std::process::Command::new("qs")
        .args(["ipc", "call", "menu", "toggle"])
        .spawn();
});
```

### Key Points
- Uses `qs ipc call menu toggle` - matches IpcHandler `target: "menu"` and `function toggle()`
- Non-blocking spawn (fire-and-forget)
- No direct GTK-Quickshell coupling needed

---

## 8. Configuration Structure

```
~/.config/quickshell/
└── niri-menu/
    ├── shell.qml      # Main entry point
    ├── LeftMenu.qml   # Left sidebar menu (this report)
    └── TopBar.qml     # Top bar (separate component)
```

### shell.qml
```qml
import QtQuick
import Quickshell

Item {
    LeftMenu { id: leftMenu }
    TopBar { id: topBar }
}
```

---

## 9. Testing Commands

```bash
# Start quickshell
qs -c niri-menu

# Test IPC (in another terminal)
qs ipc call menu toggle
qs ipc call menu show
qs ipc call menu hide
qs ipc show  # Lists registered targets

# Wait for signal
qs ipc wait menu opened
```

---

## 10. Known Issues / Warnings

| Warning | Cause | Fix |
|---------|-------|-----|
| `Setting width is deprecated` | Using `width`/`height` on PanelWindow | Use `implicitWidth`/`implicitHeight` |
| `Unable to assign [undefined] to QString` | `image://theme/` provider not available | Use text/icons instead of theme images |
| `Cannot assign to read-only property onSignalTriggered` | Trying to override C++ method | Use QML function definitions on IpcHandler directly |

---

## 11. Architecture Summary

```
┌─────────────────────────────────────────────────────────────┐
│                      Rust Bar (GTK)                         │
│  [Grid Icon Button] ──qs ipc call menu toggle──►            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Quickshell (QML)                         │
│  ┌─────────────┐    ┌───────────────────────────────────┐  │
│  │ IpcHandler  │───►│  LeftMenu (PanelWindow)           │  │
│  │ target:menu │    │  • SpringAnimation (x position)   │  │
│  │ function    │    │  • BlinkFlash (SequentialAnimation)│  │
│  │   toggle()  │    │  • Backdrop fade                  │  │
│  └─────────────┘    └───────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 12. Next Steps / Extensions

1. **Add workspace list** - Connect to niri IPC for dynamic workspace buttons
2. **Application launcher** - Integrate with `gio` or `.desktop` file parsing
3. **Settings panel** - Sub-menu for bar configuration
4. **Power menu** - Logout/reboot/shutdown via systemd/loginctl
5. **Keyboard navigation** - Arrow keys, Enter, Escape support
6. **Search/filter** - Type-to-filter applications

---

## References

- [Quickshell IpcHandler Docs](https://quickshell.org/docs/v0.3.1/types/Quickshell.Io/IpcHandler/)
- [Quickshell PanelWindow](https://quickshell.org/docs/v0.3.1/types/Quickshell._Window/PanelWindow/)
- [SpringAnimation Qt Docs](https://doc.qt.io/qt-6/qml-qtquick-springanimation.html)
- [Quickshell GitHub](https://github.com/quickshell/quickshell)