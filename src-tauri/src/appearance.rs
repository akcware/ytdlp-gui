//! Windows 11 look: Mica window backdrop and the user's accent color.
//!
//! The window is created transparent and hidden (see tauri.conf.json). On
//! Windows 11 we apply Mica; the frontend asks for the result before showing
//! the window, so it can drop its solid background only when Mica is really
//! behind it (Windows 10 keeps the solid Fluent background).

use std::sync::OnceLock;

use serde::Serialize;
use tauri::WebviewWindow;
use tauri::window::{Effect, EffectsBuilder};

use crate::process::blocking;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Appearance {
    /// "mica" when the Mica backdrop was applied, otherwise "none".
    backdrop: &'static str,
    accent: Accent,
}

/// Shades of the system accent color as `#rrggbb`. Fluent uses `dark1` for
/// accent fills on the light theme and `light2` on the dark theme.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Accent {
    light3: String,
    light2: String,
    light1: String,
    base: String,
    dark1: String,
    dark2: String,
    dark3: String,
}

impl Default for Accent {
    /// Windows 11 default blue.
    fn default() -> Self {
        Self {
            light3: "#99ebff".into(),
            light2: "#60cdff".into(),
            light1: "#00b7ff".into(),
            base: "#0078d4".into(),
            dark1: "#005fb8".into(),
            dark2: "#003e92".into(),
            dark3: "#001a68".into(),
        }
    }
}

static APPEARANCE: OnceLock<Appearance> = OnceLock::new();

/// Applies Mica (once) and returns the cached appearance. Called from a
/// background thread at startup and from the `appearance` command, whichever
/// comes first.
pub fn init(window: &WebviewWindow) -> &'static Appearance {
    APPEARANCE.get_or_init(|| {
        let mica = is_windows_11()
            && window
                .set_effects(EffectsBuilder::new().effect(Effect::Mica).build())
                .is_ok();
        Appearance {
            backdrop: if mica { "mica" } else { "none" },
            accent: read_accent().unwrap_or_default(),
        }
    })
}

#[tauri::command]
pub async fn appearance(window: WebviewWindow) -> Result<Appearance, String> {
    blocking(move || Ok(init(&window).clone())).await
}

/// Reads one registry value with reg.exe (no extra crates needed).
/// Returns `(type, data)`, e.g. `("REG_SZ", "26200")`.
#[cfg(windows)]
fn reg_value(key: &str, name: &str) -> Option<(String, String)> {
    let out = crate::process::hidden_command("reg")
        .args(["query", key, "/v", name])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).lines().find_map(|line| {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        (tokens.len() >= 3 && tokens[0].eq_ignore_ascii_case(name))
            .then(|| (tokens[1].to_string(), tokens[2].to_string()))
    })
}

#[cfg(windows)]
fn is_windows_11() -> bool {
    reg_value(
        r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "CurrentBuildNumber",
    )
    .and_then(|(_, build)| build.parse::<u32>().ok())
    .is_some_and(|build| build >= 22000)
}

#[cfg(not(windows))]
fn is_windows_11() -> bool {
    false
}

/// `AccentPalette` holds 8 RGBA colors: Light3, Light2, Light1, Base, Dark1,
/// Dark2, Dark3 and an unused one.
#[cfg(windows)]
fn read_accent() -> Option<Accent> {
    let (kind, hex) = reg_value(
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent",
        "AccentPalette",
    )?;
    if kind != "REG_BINARY" || hex.len() < 56 || !hex.is_ascii() {
        return None;
    }
    let bytes: Vec<u8> = (0..hex.len() / 2)
        .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16))
        .collect::<Result<_, _>>()
        .ok()?;
    let color = |i: usize| format!("#{:02x}{:02x}{:02x}", bytes[i * 4], bytes[i * 4 + 1], bytes[i * 4 + 2]);
    Some(Accent {
        light3: color(0),
        light2: color(1),
        light1: color(2),
        base: color(3),
        dark1: color(4),
        dark2: color(5),
        dark3: color(6),
    })
}

#[cfg(not(windows))]
fn read_accent() -> Option<Accent> {
    None
}
