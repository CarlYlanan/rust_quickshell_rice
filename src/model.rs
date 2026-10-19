use serde::Serialize;

pub const CYAN: &str = "#00d0d0";
pub const PINK: &str = "#ff4d6d";
pub const NAVY: &str = "#1a1f2e";
pub const PALE_RING: &str = "#d9d9d9";
pub const WHITE_FILL: &str = "#f4f7fb";
pub const SHADOW: &str = "#0a0d18";

pub const WS_COLORS: [&str; 5] = [CYAN, PINK, "#ffe066", "#c0b0ff", "#a0ffc8"];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Workspace {
    pub id: u8,
    pub active: bool,
    pub focused: bool,
    pub occupied: bool,
}

impl Workspace {
    pub const fn kanji(self) -> char {
        match self.id {
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
        }
    }

    pub fn ws_color(self) -> &'static str {
        WS_COLORS[((self.id.saturating_sub(1)) % 5) as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Volume {
    pub percent: u32,
    pub muted: bool,
}

impl Volume {
    pub fn read() -> Option<Self> {
        use std::process::Command;
        let output = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut parts = stdout.split_whitespace();
        parts.next();
        let vol_str = parts.next()?;
        let percent = (vol_str.parse::<f32>().ok()? * 100.0).round() as u32;
        let muted = stdout.contains("[MUTED]");
        Some(Self { percent, muted })
    }
}