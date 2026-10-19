#!/usr/bin/env bash

# Folder where your art lives
WALLPAPER_DIR="/home/carly/Pictures/git/Desktop-Config/wallpapers"

# 1. Clean up Monitor Names
# Grabs the names from swww query, ignoring the leading colon
MONITOR_LIST=$(awww query | awk -F: '{print $2}' | xargs)

# 2. Step One: Select Display
# Wide menu to match your 986px preference
SELECTED_MONITOR=$(echo -e "${MONITOR_LIST// /\\n}" | rofi -dmenu \
    -p "Select Display" \
    -theme-str "
        window { width: 986px; border: 2px; border-color: #00BEBE; border-radius: 15px; background-color: #0F1117; }
        listview { lines: 5; padding: 20px; spacing: 10px; }
        element { padding: 15px; border-radius: 10px; background-color: #1A1B26; }
        element-text { text-color: #ECEFF4; font: \"JetBrainsMono Nerd Font 14\"; horizontal-align: 0.5; }
        element selected { background-color: #00BEBE; }
        element-text selected { text-color: #0F1117; font: \"JetBrainsMono Nerd Font 14 Bold\"; }
    ")

# Exit if no monitor was picked
if [ -z "$SELECTED_MONITOR" ]; then exit; fi

# 3. Build Wallpaper List with Thumbnails
list_items=""
for file in "$WALLPAPER_DIR"/*.{jpg,jpeg,png,webp}; do
    if [ -f "$file" ]; then
        basename=$(basename "$file")
        list_items+="${basename}\0icon\x1f${file}\n"
    fi
done

# 4. Step Two: Select Wallpaper (Vertical 3-Column Grid)
choice=$(echo -e "$list_items" | rofi -dmenu \
    -i \
    -p "Target: $SELECTED_MONITOR" \
    -show-icons \
    -theme-str "
        window {
            width: 986px;
            border: 2px;
            border-color: #FF4D6D;
            border-radius: 15px;
            background-color: #0F1117;
        }
        mainbox {
            orientation: vertical;
            children: [ \"inputbar\", \"listview\" ];
        }
        inputbar {
            padding: 15px;
            background-color: #1A1B26;
            children: [ \"prompt\" ];
        }
        prompt {
            text-color: #00BEBE;
            font: \"JetBrainsMono Nerd Font 12 Bold\";
        }
        listview {
            columns: 3;
            lines: 3;
            spacing: 20px;
            padding: 20px;
            fixed-columns: true;
            fixed-height: true;
            scrollbar: true;
            layout: vertical;
        }
        element {
            orientation: vertical;
            padding: 15px;
            border-radius: 10px;
            background-color: #1A1B26;
        }
        element-icon {
            size: 200px;
            horizontal-align: 0.5;
        }
        element-text {
            horizontal-align: 0.5;
            text-color: #ECEFF4;
            padding: 10px 0 0 0;
            font: \"JetBrainsMono Nerd Font 10\";
        }
        element selected {
            background-color: #313244;
            border: 2px;
            border-color: #00BEBE;
        }
        element-text selected {
            text-color: #00BEBE;
        }
        scrollbar {
            width: 4px;
            handle-width: 8px;
            handle-color: #00BEBE;
        }
    ")

# 5. Apply the choice
if [ -n "$choice" ]; then
    FULL_PATH="$WALLPAPER_DIR/$choice"

    # Apply to the specific monitor with the grow transition
    awww img -o "$SELECTED_MONITOR" "$FULL_PATH" --transition-type grow --transition-pos 0.85,0.97 --transition-duration 1.5

    # Save the path to cache for boot persistence
    echo "$FULL_PATH" > "$HOME/.cache/.last_wallpaper_$SELECTED_MONITOR"
fi
