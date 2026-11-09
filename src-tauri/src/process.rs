//! Helpers for spawning external programs (yt-dlp, curl, tar, ...).

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

/// Builds a `Command` that never opens a console window on Windows and asks
/// Python (yt-dlp is a Python app) to talk UTF-8 over its pipes.
pub fn hidden_command(program: impl AsRef<OsStr>) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.env("PYTHONIOENCODING", "utf-8").env("PYTHONUTF8", "1");
    cmd
}

/// Path to a program that ships with Windows (curl.exe, tar.exe, ...).
/// Using the full System32 path avoids picking up e.g. Git's GNU tar, which
/// cannot extract zip files.
#[cfg(windows)]
pub fn system32(exe: &str) -> PathBuf {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    PathBuf::from(root).join("System32").join(exe)
}

#[cfg(not(windows))]
pub fn system32(exe: &str) -> PathBuf {
    PathBuf::from(exe.trim_end_matches(".exe"))
}

/// Runs blocking work (spawning processes, waiting on downloads) off the main
/// thread so the window stays responsive.
pub async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

/// Kills a process together with its children (yt-dlp spawns ffmpeg, and the
/// Windows yt-dlp.exe is itself a launcher process plus a Python child).
pub fn kill_tree(pid: u32) {
    #[cfg(windows)]
    let _ = hidden_command("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output();
    #[cfg(not(windows))]
    let _ = Command::new("kill").arg(pid.to_string()).output();
}
