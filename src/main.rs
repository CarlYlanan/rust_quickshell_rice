use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Local;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use serde_json::Value;

use owo_bar::system::{WorkspaceStream, workspace_states_for_output};
use status_notifier::{Host as StatusNotifierHost, Item as TrayItem};

const BAR_HEIGHT: i32 = 42;

struct Shell {
    monitors: Vec<MonitorUi>,
    workspace_stream: Option<WorkspaceStream>,
    tray_host: Option<Rc<RefCell<StatusNotifierHost>>>,
}

struct MonitorUi {
    output: String,
    monitor: gdk::Monitor,
    bar: gtk::Window,
    alive: Rc<Cell<bool>>,
    workspace_label: gtk::Label,
    workspace_buttons_box: gtk::Box,
    clock_button: gtk::Button,
    tray_box: gtk::Box,
    volume_button: gtk::Button,
    _tray_host: Option<Rc<RefCell<StatusNotifierHost>>>,
}

pub fn run() {
    let application = gtk::Application::builder()
        .application_id("dev.owo.bar")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    let shell = Rc::new(std::cell::RefCell::new(None::<Shell>));

    let shell_for_startup = shell.clone();
    application.connect_startup(move |app| {
        install_theme();
        *std::cell::RefCell::<_>::borrow_mut(&shell_for_startup) = Some(build_shell(app));
    });

    application.connect_command_line(move |_app, cmd| {
        let args = cmd.arguments();
        let action = args.get(1).and_then(|a| a.to_str()).unwrap_or("bar");
        if let Some(shell) = shell.borrow().as_ref() {
            match action {
                "bar" => for ui in &shell.monitors { ui.bar.present(); },
                _ => eprintln!("Usage: owo-bar [bar]"),
            }
            glib::ExitCode::SUCCESS
        } else {
            glib::ExitCode::FAILURE
        }
    });

    let args: Vec<String> = std::env::args().collect();
    application.run_with_args(&args);
}

fn build_shell(application: &gtk::Application) -> Shell {
    let workspace_stream = WorkspaceStream::new();
    let workspace_snapshot = workspace_stream.as_ref().map(|s| s.snapshot());

    let tray_host = StatusNotifierHost::new()
        .map(|h| Rc::new(RefCell::new(h)))
        .map_err(|e| eprintln!("Tray host failed: {e}"))
        .ok();

    let mut monitors = Vec::new();
    for monitor in display_monitors() {
        let output = output_name_for_monitor(&monitor);
        if let Some(ui) = build_monitor_ui(
            application,
            &monitor,
            &output,
            &workspace_snapshot,
            &tray_host,
        ) {
            monitors.push(ui);
        }
    }
    for ui in &monitors { ui.bar.present(); }

    Shell { monitors, workspace_stream, tray_host }
}

fn build_monitor_ui(
    application: &gtk::Application,
    monitor: &gdk::Monitor,
    output: &str,
    workspace_snapshot: &Option<Arc<Mutex<owo_bar::system::WorkspaceSnapshot>>>,
    tray_host: &Option<Rc<RefCell<StatusNotifierHost>>>,
) -> Option<MonitorUi> {
    let alive = Rc::new(Cell::new(true));
    let geometry = monitor.geometry();

    let window = gtk::Window::builder()
        .application(application)
        .title("owo-bar")
        .decorated(false)
        .build();

    window.init_layer_shell();
    window.set_namespace(Some("owo-bar"));
    window.set_widget_name("owo-bar");
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_monitor(Some(monitor));
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 4);
    window.set_margin(Edge::Left, 10);
    window.set_margin(Edge::Right, 10);
    window.auto_exclusive_zone_enable();
    window.set_default_size(geometry.width(), BAR_HEIGHT);

    let output_name = output.to_string();

    // Workspace label (original style) - empty text but keeps width
    let workspace_label = gtk::Label::new(None);
    workspace_label.add_css_class("workspace-label");
    workspace_label.set_valign(gtk::Align::Center);
    workspace_label.set_halign(gtk::Align::Center);
    // Set minimum width to match "WORKSPACE 9" (longest possible)
    workspace_label.set_width_chars(12); // "WORKSPACE 9" = 11 chars + padding

    // Container for workspace buttons (kanji) - one button per workspace 1-9
    let workspace_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    workspace_buttons_box.set_valign(gtk::Align::Center);
    workspace_buttons_box.set_halign(gtk::Align::Center);
    // We'll populate this in refresh_workspaces; for now empty.

    let clock_button = gtk::Button::with_label("");
    clock_button.add_css_class("clock-button");
    clock_button.set_valign(gtk::Align::Center);
    clock_button.set_halign(gtk::Align::Center);
    update_clock(&clock_button);

    let tray_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    tray_box.set_valign(gtk::Align::Center);
    tray_box.add_css_class("tray-group");

    // Volume button
    let volume_button = gtk::Button::with_label("");
    volume_button.add_css_class("volume-button");
    volume_button.set_valign(gtk::Align::Center);
    volume_button.set_halign(gtk::Align::Center);
    update_volume(&volume_button);
    volume_button.connect_clicked(move |_| {
        let _ = std::process::Command::new("pavucontrol").spawn();
    });

    // Rotate image button
    let rotate_button = gtk::Button::new();
    rotate_button.add_css_class("rotate-button");
    rotate_button.set_valign(gtk::Align::Center);
    rotate_button.set_halign(gtk::Align::Center);
    let rotate_img = gtk::Image::from_file("/home/cylanan/Documents/one_last_time/0w0/images/smolteto.png");
    rotate_img.set_pixel_size(24);
    rotate_button.set_child(Some(&rotate_img));
    rotate_button.connect_clicked(move |_| {
        let _ = std::process::Command::new("/home/cylanan/Documents/one_last_time/0w0/scripts/rotate.sh").spawn();
    });

    // Tray container with volume and rotate button
    // Waybar tray spacing: 10, modules-right module margin: 5px each side
    let tray_container = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    tray_container.set_valign(gtk::Align::Center);
    tray_container.append(&tray_box);
    tray_container.append(&volume_button);
    tray_container.append(&rotate_button);

    // Create pill containers for each module
    // Workspace label pill (original style)
    let label_pill = gtk::Box::new(gtk::Orientation::Vertical, 0);
    label_pill.add_css_class("pill");
    label_pill.set_hexpand(true);
    label_pill.set_valign(gtk::Align::Center);
    label_pill.append(&workspace_label);

    // Workspace kanji pill
    let kanji_pill = gtk::Box::new(gtk::Orientation::Vertical, 0);
    kanji_pill.add_css_class("pill");
    kanji_pill.set_hexpand(true);
    kanji_pill.set_valign(gtk::Align::Center);
    kanji_pill.append(&workspace_buttons_box);

    // Workspace container (holds label pill and kanji pill)
    let workspace_container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    workspace_container.set_halign(gtk::Align::Start);
    workspace_container.set_valign(gtk::Align::Center);
    workspace_container.append(&label_pill);
    workspace_container.append(&kanji_pill);

    let clock_pill = gtk::Box::new(gtk::Orientation::Vertical, 0);
    clock_pill.add_css_class("pill");
    clock_pill.set_halign(gtk::Align::Center);
    clock_pill.set_valign(gtk::Align::Center);
    clock_pill.append(&clock_button);

    let tray_pill = gtk::Box::new(gtk::Orientation::Vertical, 0);
    tray_pill.add_css_class("pill");
    tray_pill.set_halign(gtk::Align::End);
    tray_pill.set_valign(gtk::Align::Center);
    tray_pill.append(&tray_container);

    // Use CenterBox directly for proper three-zone layout
    let layout = gtk::CenterBox::new();
    layout.add_css_class("bar-layout");
    layout.set_size_request(-1, BAR_HEIGHT);
    layout.set_hexpand(true);
    layout.set_start_widget(Some(&workspace_container));
    layout.set_center_widget(Some(&clock_pill));
    layout.set_end_widget(Some(&tray_pill));

    window.set_child(Some(&layout));

    if let Some(host) = tray_host.clone() {
        let tray_box_clone = tray_box.clone();
        let cached = Rc::new(std::cell::RefCell::new(Vec::<(String, TrayItem)>::new()));
        let alive_tray = alive.clone();
        let bar_clone = window.clone();
        let geom = geometry;
        glib::timeout_add_local(Duration::from_millis(250), move || {
            if !alive_tray.get() { return glib::ControlFlow::Break; }
            refresh_tray(&tray_box_clone, &cached, &host, &bar_clone, geom.x(), geom.y());
            glib::ControlFlow::Continue
        });
    }

    // Workspace updates (only updates kanji buttons, label removed)
    let alive_ws = alive.clone();
    let output_clone = output_name.clone();
    let workspace_buttons_clone = workspace_buttons_box.clone();
    if let Some(snap) = workspace_snapshot.clone() {
        let last_rev = Rc::new(Cell::new(0_u64));
        let buttons_clone = workspace_buttons_clone.clone();
        glib::timeout_add_local(Duration::from_millis(50), move || {
            if !alive_ws.get() { return glib::ControlFlow::Break; }
            if let Ok(s) = snap.lock() {
                if s.revision != last_rev.get() {
                    refresh_workspaces(&buttons_clone, output_clone.clone(), &s.workspaces);
                    last_rev.set(s.revision);
                }
            }
            glib::ControlFlow::Continue
        });
    } else {
        let buttons_clone = workspace_buttons_clone.clone();
        glib::timeout_add_seconds_local(1, move || {
            if !alive_ws.get() { return glib::ControlFlow::Break; }
            let ws = owo_bar::system::refresh_workspaces_from_niri(&output_clone);
            refresh_workspaces(&buttons_clone, output_clone.clone(), &ws.into_iter().map(|w| serde_json::to_value(w).unwrap()).collect::<Vec<_>>());
            glib::ControlFlow::Continue
        });
    }

    // Clock updates
    let alive_clock = alive.clone();
    let clock_clone = clock_button.clone();
    glib::timeout_add_seconds_local(1, move || {
        if !alive_clock.get() { return glib::ControlFlow::Break; }
        update_clock(&clock_clone);
        glib::ControlFlow::Continue
    });

    // Volume updates
    let alive_volume = alive.clone();
    let volume_clone = volume_button.clone();
    glib::timeout_add_local(Duration::from_millis(500), move || {
        if !alive_volume.get() { return glib::ControlFlow::Break; }
        update_volume(&volume_clone);
        glib::ControlFlow::Continue
    });

    Some(MonitorUi {
        output: output_name,
        monitor: monitor.clone(),
        bar: window,
        alive,
        workspace_label,
        workspace_buttons_box,
        clock_button,
        tray_box,
        volume_button,
        _tray_host: tray_host.clone(),
    })
}

fn update_clock(button: &gtk::Button) {
    let now = Local::now();
    button.set_label(&now.format("%m月%d日 %H:%M").to_string());
}

fn update_volume(button: &gtk::Button) {
    let output = std::process::Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output();
    let label = match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut parts = stdout.split_whitespace();
            parts.next(); // skip "Volume:"
            if let Some(vol_str) = parts.next() {
                if let Ok(vol) = vol_str.parse::<f32>() {
                    let percent = (vol * 100.0).round() as u32;
                    let muted = stdout.contains("[MUTED]");
                    // Waybar-style pulseaudio icons:  (up),  (down),  (off),  (mute)
                    let icon = if muted {
                        ""  // fa-volume-mute
                    } else if percent > 66 {
                        ""  // fa-volume-up
                    } else if percent > 33 {
                        ""  // fa-volume-down
                    } else {
                        ""  // fa-volume-off
                    };
                    if muted {
                        format!("{} {}", icon, percent)
                    } else {
                        format!("{} {}%", icon, percent)
                    }
                } else {
                    "?%".to_string()
                }
            } else {
                "?%".to_string()
            }
        }
        _ => "?%".to_string(),
    };
    button.set_label(&label);
}

fn display_monitors() -> Vec<gdk::Monitor> {
    gdk::Display::default()
        .map(|d| {
            (0..d.monitors().n_items())
                .filter_map(|i| d.monitors().item(i).and_then(|m| m.downcast::<gdk::Monitor>().ok()))
                .collect()
        })
        .unwrap_or_default()
}

fn output_name_for_monitor(monitor: &gdk::Monitor) -> String {
    if let Some(c) = monitor.connector() { return c.to_string(); }
    let Ok(out) = Command::new("niri").args(["msg", "--json", "outputs"]).output() else { return String::new() };
    let Ok(outputs) = serde_json::from_slice::<Value>(&out.stdout) else { return String::new() };
    let Some(obj) = outputs.as_object() else { return String::new() };
    let geo = monitor.geometry();
    obj.iter()
        .find(|(_, v)| {
            v.get("logical")
                .and_then(|l| l.get("x")).and_then(Value::as_f64) == Some(f64::from(geo.x()))
            && v.get("logical")
                .and_then(|l| l.get("y")).and_then(Value::as_f64) == Some(f64::from(geo.y()))
        })
        .map(|(n, _)| n.clone())
        .unwrap_or_default()
}

fn refresh_workspaces(buttons_box: &gtk::Box, output_name: String, workspaces: &[Value]) {
    let states = workspace_states_for_output(&output_name, workspaces);
    // Determine ACTIVE workspace id (workspace currently on this monitor)
    let active_id = states.iter().find(|s| s.active).map(|s| s.id).unwrap_or(0);

    // Clear existing buttons
    while let Some(child) = buttons_box.first_child() {
        buttons_box.remove(&child);
    }

    // Get all occupied workspaces for this monitor
    let mut occupied_states: Vec<_> = states.iter().filter(|s| s.occupied).collect();
    occupied_states.sort_by_key(|s| s.id);

    // Determine which workspaces to show:
    // - All occupied workspaces
    // - PLUS the next workspace after the highest occupied (or active) as a placeholder
    let mut ids_to_show: Vec<u8> = occupied_states.iter().map(|s| s.id).collect();

    // Find the highest occupied workspace, or fall back to active_id
    let highest_occupied = occupied_states.last().map(|s| s.id).unwrap_or(active_id);

    // Add next workspace as placeholder if it exists (1-9) and isn't already shown
    let next_id = if highest_occupied >= 9 { 1 } else { highest_occupied + 1 };
    if !ids_to_show.contains(&next_id) {
        ids_to_show.push(next_id);
    }

    // Create buttons for all selected workspace IDs
    for id in &ids_to_show {
        let is_occupied = states.iter().any(|s| s.id == *id && s.occupied);
        let is_active = states.iter().any(|s| s.id == *id && s.active);

        let btn = create_workspace_button(*id, output_name.clone());

        if is_active {
            btn.add_css_class("active");
        }
        if !is_occupied {
            btn.add_css_class("placeholder");
        }

        buttons_box.append(&btn);
    }

    // Apply focused state to the FOCUSED workspace button (for visual highlight)
    let focused_id = states.iter().find(|s| s.focused).map(|s| s.id).unwrap_or(0);
    if focused_id >= 1 && focused_id <= 9 {
        let target_name = format!("ws-{}", focused_id);
        let mut child = buttons_box.first_child();
        while let Some(w) = child {
            if let Ok(btn) = w.clone().downcast::<gtk::Button>() {
                if btn.widget_name() == target_name {
                    btn.add_css_class("focused");
                    break;
                }
            }
            child = w.next_sibling();
        }
    }
}

fn create_workspace_button(id: u8, output_name: String) -> gtk::Button {
    let btn = gtk::Button::with_label("");
    btn.add_css_class("workspace-btn");
    btn.set_valign(gtk::Align::Center);
    btn.set_halign(gtk::Align::Center);
    // Add class for color based on id (1-9, wrapping every 5)
    let color_id = ((id - 1) % 5) + 1;
    btn.add_css_class(&format!("ws-{}", color_id));
    // Set widget name for identification
    btn.set_widget_name(&format!("ws-{}", id));
    // Set label to kanji
    let kanji = match id {
        1 => '一',
        2 => '二',
        3 => '三',
        4 => '四',
        5 => '五',
        6 => '六',
        7 => '七',
        8 => '八',
        9 => '九',
        _ => '〇',
    };
    btn.set_label(&kanji.to_string());
    // Tooltip: show workspace id
    btn.set_tooltip_text(Some(&format!("Workspace {}", id)));
    // Click handler
    let output = output_name.clone();
    btn.connect_clicked(move |_b| {
        let out = output.clone();
        std::thread::spawn(move || {
            let ok = out.is_empty() || Command::new("niri").args(["msg", "action", "focus-monitor", &out]).status().is_ok_and(|s| s.success());
            if ok { let _ = Command::new("niri").args(["msg", "action", "focus-workspace", &id.to_string()]).spawn(); }
        });
    });
    btn
}

fn tray_items_equal(a: &[(String, TrayItem)], b: &[(String, TrayItem)]) -> bool {
    if a.len() != b.len() { return false; }
    for (ai, bi) in a.iter().zip(b.iter()) {
        if ai.0 != bi.0 { return false; }
        let (_, aitem) = ai;
        let (_, bitem) = bi;
        // Compare only stable identifying fields, not pixmap data which changes frequently
        if aitem.bus_name != bitem.bus_name
            || aitem.id != bitem.id
            || aitem.title != bitem.title
            || aitem.status != bitem.status
            || aitem.icon_name != bitem.icon_name
            || aitem.attention_icon_name != bitem.attention_icon_name
            || aitem.overlay_icon_name != bitem.overlay_icon_name
            || aitem.category != bitem.category
            || aitem.item_is_menu != bitem.item_is_menu
        {
            return false;
        }
    }
    true
}

fn refresh_tray(
    list: &gtk::Box,
    cached: &Rc<std::cell::RefCell<Vec<(String, TrayItem)>>>,
    host: &Rc<RefCell<StatusNotifierHost>>,
    bar: &gtk::Window,
    mon_x: i32,
    mon_y: i32,
) {
    let items = {
        let mut h = host.borrow_mut();
        // Process events but suppress errors to avoid spam
        let _ = h.process_events(|_,_,_|{}, |_e| {}, Some(Duration::ZERO));
        let all_items = h.get_all_items()
            .map(|(n, i)| (n.clone(), i.clone()))
            .collect::<Vec<_>>();

        all_items
    };
    if tray_items_equal(&*cached.borrow(), &items) { return; }
    while let Some(c) = list.first_child() { list.remove(&c); }
    for (bus, item) in &items {
        let btn = gtk::Button::new();
        btn.add_css_class("tray-item");
        btn.set_valign(gtk::Align::Center);
        btn.set_halign(gtk::Align::Center);
        btn.set_can_target(true);
        btn.set_focus_on_click(false);
        if item.status == "NeedsAttention" { btn.add_css_class("needs-attention"); }
        let tooltip = if !item.tool_tip.title.is_empty() {
            if item.tool_tip.description.is_empty() { item.tool_tip.title.clone() }
            else { format!("{}\n{}", item.tool_tip.title, item.tool_tip.description) }
        } else if item.title.is_empty() { item.id.clone() } else { item.title.clone() };
        btn.set_tooltip_text(Some(&tooltip));
        let icon_img = tray_icon(item);
        btn.set_child(Some(&icon_img));

        // Use button's native clicked signal for LEFT click (activate)
        let host_c_left = host.clone();
        let bus_c_left = bus.clone();
        let is_menu_left = item.item_is_menu;
        let bar_c_left = bar.clone();
        let btn_c_left = btn.clone();
        let mon_x_left = mon_x;
        let mon_y_left = mon_y;
        btn.connect_clicked(move |_| {
            let bounds = btn_c_left.compute_bounds(&bar_c_left);
            let (sx, sy) = bounds.map_or((mon_x_left, mon_y_left), |b| (
                mon_x_left + (b.x() + b.width() / 2.0).round() as i32,
                mon_y_left + (b.y() + b.height() / 2.0).round() as i32
            ));
            let mut h = host_c_left.borrow_mut();
            let r = if is_menu_left { h.item_context_menu(&bus_c_left, sx, sy) } else { h.item_activate(&bus_c_left, sx, sy) };
            if let Err(e) = r { eprintln!("tray left-click {bus_c_left}: {e}"); }
        });

        // Use gesture for RIGHT click (context menu) and MIDDLE click
        let host_c_right = host.clone();
        let bus_c_right = bus.clone();
        let bar_c_right = bar.clone();
        let btn_c_right = btn.clone();
        let mon_x_right = mon_x;
        let mon_y_right = mon_y;
        let ges = gtk::GestureClick::new();
        ges.set_button(0);
        ges.connect_pressed(move |g, _, x, y| {
            let button = g.current_button();
            if button == 1 { return; } // left click handled by button clicked

            let bounds = btn_c_right.compute_bounds(&bar_c_right);
            let (sx, sy) = bounds.map_or((mon_x_right, mon_y_right), |b| (
                mon_x_right + (b.x() + x as f32).round() as i32,
                mon_y_right + (b.y() + y as f32).round() as i32
            ));
            let mut h = host_c_right.borrow_mut();
            let r = match button {
                2 => h.item_secondary_activate(&bus_c_right, sx, sy),
                3 => h.item_context_menu(&bus_c_right, sx, sy),
                _ => return,
            };
            if let Err(e) = r { eprintln!("tray right/middle {bus_c_right}: {e}"); }
        });
        btn.add_controller(ges);
        list.append(&btn);
    }
    *cached.borrow_mut() = items;
}

fn tray_icon(item: &TrayItem) -> gtk::Image {
    // Try pixmaps first (attention > normal)
    let pixmaps = if item.status == "NeedsAttention" && !item.attention_icon_pixmap.is_empty() {
        &item.attention_icon_pixmap
    } else { &item.icon_pixmap };

    // Try largest first for better quality
    for p in pixmaps.iter().rev() {
        let [w, h] = p.dimensions;
        let exp = w.max(0) as usize * h.max(0) as usize * 4;
        if w > 0 && h > 0 && p.image_data.len() == exp {
            // Check if pixmap has any non-transparent pixels
            let has_visible = p.image_data.chunks_exact(4).any(|px| px[0] > 0); // alpha channel
            if has_visible {
                let mut bgra = Vec::with_capacity(exp);
                for px in p.image_data.as_chunks::<4>().0 {
                    bgra.extend_from_slice(&[px[3], px[2], px[1], px[0]]);
                }
                let bytes = glib::Bytes::from_owned(bgra);
                let tex = gdk::MemoryTexture::new(w, h, gdk::MemoryFormat::B8g8r8a8, &bytes, w as usize * 4);
                let img = gtk::Image::from_paintable(Some(&tex));
                img.set_pixel_size(20);
                return img;
            }
        }
    }

    // Build fallback icon name chain
    let mut candidates = Vec::new();
    if item.status == "NeedsAttention" && !item.attention_icon_name.is_empty() {
        candidates.push(item.attention_icon_name.clone());
    }
    if !item.icon_name.is_empty() {
        candidates.push(item.icon_name.clone());
    }
    if !item.id.is_empty() {
        candidates.push(item.id.clone());
    }
    // Common app-specific fallbacks
    match item.id.as_str() {
        "obs" => candidates.extend(["obs-studio".to_string(), "camera-video".to_string()]),
        "discord" => candidates.extend(["discord".to_string(), "chat".to_string()]),
        _ => {}
    }
    candidates.push("camera-video".to_string()); // ultimate fallback

    // Try each candidate
    for name in candidates {
        let img = gtk::Image::from_icon_name(&name);
        if img.paintable().is_some() {
            img.set_pixel_size(20);
            return img;
        }
    }

    // Last resort
    let img = gtk::Image::from_icon_name("camera-video");
    img.set_pixel_size(20);
    img
}

fn install_theme() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("../theme.css"));
    if let Some(d) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&d, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

fn main() {
    run();
}