//! Manages the yt-dlp and ffmpeg binaries in the app's local data folder
//! (`%LOCALAPPDATA%\com.kadir.ytdlpgui\bin` on Windows).
//!
//! They are downloaded on first run instead of being bundled: the installer
//! stays small and yt-dlp can update itself (`yt-dlp -U`) without a new app
//! release.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::process::{blocking, hidden_command, system32};

const EXE: &str = if cfg!(windows) { ".exe" } else { "" };

#[cfg(windows)]
const YTDLP_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
#[cfg(target_os = "macos")]
const YTDLP_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos";
#[cfg(all(unix, not(target_os = "macos")))]
const YTDLP_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux";

/// ffmpeg builds maintained by the yt-dlp team (static, includes ffprobe).
#[cfg(windows)]
const FFMPEG_URL: &str = "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    /// `None` when yt-dlp is not installed (or does not run).
    pub ytdlp_version: Option<String>,
    /// ffmpeg found in our bin folder or on PATH.
    pub ffmpeg: bool,
    pub bin_dir: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ToolProgress {
    tool: String,
    /// "download" or "extract"
    phase: &'static str,
    bytes: u64,
}

pub fn bin_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("bin");
    fs::create_dir_all(&dir).map_err(|e| format!("Couldn't create the folder: {e}"))?;
    Ok(dir)
}

pub fn ytdlp_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(bin_dir(app)?.join(format!("yt-dlp{EXE}")))
}

/// Our own ffmpeg copy, if one was installed through the app.
pub fn local_ffmpeg(app: &AppHandle) -> Option<PathBuf> {
    let path = bin_dir(app).ok()?.join(format!("ffmpeg{EXE}"));
    path.exists().then_some(path)
}

pub fn ffmpeg_available(app: &AppHandle) -> bool {
    local_ffmpeg(app).is_some()
        || hidden_command("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
}

fn ytdlp_version(app: &AppHandle) -> Option<String> {
    let path = ytdlp_path(app).ok()?;
    if !path.exists() {
        return None;
    }
    let out = hidden_command(&path).arg("--version").output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn status(app: &AppHandle) -> Result<ToolStatus, String> {
    Ok(ToolStatus {
        ytdlp_version: ytdlp_version(app),
        ffmpeg: ffmpeg_available(app),
        bin_dir: bin_dir(app)?.to_string_lossy().into_owned(),
    })
}

/// Downloads `url` to `dest` with the system curl (ships with Windows 10+,
/// macOS and most Linux distros), so the app needs no HTTP/TLS crates.
/// Emits `tool-progress` events with the number of bytes received so far.
fn download_file(app: &AppHandle, url: &str, dest: &Path, tool: &str) -> Result<(), String> {
    let part = dest.with_extension("part");
    let _ = fs::remove_file(&part);

    let mut child = hidden_command(system32("curl.exe"))
        .args(["-fsSL", "--retry", "2", "-o"])
        .arg(&part)
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Couldn't start curl: {e}"))?;

    loop {
        if let Some(exit) = child.try_wait().map_err(|e| e.to_string())? {
            if !exit.success() {
                let mut err = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut err);
                }
                let _ = fs::remove_file(&part);
                return Err(format!("Couldn't download {tool}: {}", err.trim()));
            }
            break;
        }
        let bytes = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        let _ = app.emit(
            "tool-progress",
            ToolProgress {
                tool: tool.to_string(),
                phase: "download",
                bytes,
            },
        );
        thread::sleep(Duration::from_millis(250));
    }

    fs::rename(&part, dest).map_err(|e| format!("Couldn't save {tool}: {e}"))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tool_status(app: AppHandle) -> Result<ToolStatus, String> {
    blocking(move || status(&app)).await
}

#[tauri::command]
pub async fn install_ytdlp(app: AppHandle) -> Result<ToolStatus, String> {
    blocking(move || {
        let path = ytdlp_path(&app)?;
        download_file(&app, YTDLP_URL, &path, "yt-dlp")?;
        #[cfg(unix)]
        make_executable(&path)?;
        status(&app)
    })
    .await
}

/// `yt-dlp -U`: replaces the exe in place with the latest stable release.
#[tauri::command]
pub async fn update_ytdlp(app: AppHandle) -> Result<String, String> {
    blocking(move || {
        let out = hidden_command(ytdlp_path(&app)?)
            .arg("-U")
            .output()
            .map_err(|e| format!("Couldn't run yt-dlp: {e}"))?;
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        if out.status.success() {
            Ok(text.trim().to_string())
        } else {
            Err(text.trim().to_string())
        }
    })
    .await
}

#[cfg(windows)]
#[tauri::command]
pub async fn install_ffmpeg(app: AppHandle) -> Result<ToolStatus, String> {
    blocking(move || {
        let bin = bin_dir(&app)?;
        let zip = bin.join("ffmpeg.zip");
        download_file(&app, FFMPEG_URL, &zip, "ffmpeg")?;
        let _ = app.emit(
            "tool-progress",
            ToolProgress {
                tool: "ffmpeg".into(),
                phase: "extract",
                bytes: 0,
            },
        );

        // Windows 10+ ships bsdtar as tar.exe, which can extract zip files.
        let tmp = bin.join("ffmpeg-extract");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
        let out = hidden_command(system32("tar.exe"))
            .arg("-xf")
            .arg(&zip)
            .arg("-C")
            .arg(&tmp)
            .output()
            .map_err(|e| format!("Couldn't start tar: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "Couldn't extract the ffmpeg archive: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }

        // The zip contains a single folder: ffmpeg-master-latest-win64-gpl/bin/*.exe
        let inner = fs::read_dir(&tmp)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path().join("bin"))
            .find(|dir| dir.join("ffmpeg.exe").exists())
            .ok_or("ffmpeg.exe was not found in the archive")?;
        for name in ["ffmpeg.exe", "ffprobe.exe"] {
            fs::rename(inner.join(name), bin.join(name))
                .map_err(|e| format!("Couldn't move {name}: {e}"))?;
        }

        let _ = fs::remove_dir_all(&tmp);
        let _ = fs::remove_file(&zip);
        status(&app)
    })
    .await
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn install_ffmpeg(_app: AppHandle) -> Result<ToolStatus, String> {
    Err("Install ffmpeg with your package manager on this platform (e.g. brew install ffmpeg).".into())
}
