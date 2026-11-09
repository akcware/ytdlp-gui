//! Helpers for spawning external programs (yt-dlp, curl, tar, ...).

use std::ffi::OsStr;
#[cfg(target_os = "macos")]
use std::path::Path;
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
    #[cfg(target_os = "macos")]
    cmd.env("PATH", macos_path());
    cmd.env("PYTHONIOENCODING", "utf-8").env("PYTHONUTF8", "1");
    cmd
}

/// An app started from Finder or the Dock only gets `/usr/bin:/bin:/usr/sbin:/sbin`,
/// so tools installed with Homebrew or MacPorts (ffmpeg, deno...) would be
/// invisible to us and to yt-dlp. Add their usual folders. Rust also uses this
/// PATH to find the program itself when it isn't given as a full path.
#[cfg(target_os = "macos")]
fn macos_path() -> std::ffi::OsString {
    use std::sync::OnceLock;
    static PATH: OnceLock<std::ffi::OsString> = OnceLock::new();
    PATH.get_or_init(|| {
        let current = std::env::var_os("PATH").unwrap_or_default();
        let mut dirs: Vec<PathBuf> = std::env::split_paths(&current).collect();
        for extra in ["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"] {
            if !dirs.iter().any(|d| d == Path::new(extra)) {
                dirs.push(PathBuf::from(extra));
            }
        }
        for base in ["/usr/bin", "/bin", "/usr/sbin", "/sbin"] {
            if !dirs.iter().any(|d| d == Path::new(base)) {
                dirs.push(PathBuf::from(base));
            }
        }
        std::env::join_paths(dirs).unwrap_or(current)
    })
    .clone()
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

/// Puts the child in its own process group, so `kill_tree` can stop it
/// together with everything it starts (ffmpeg). No-op on Windows, where
/// taskkill /T follows the process tree instead.
pub fn own_process_group(cmd: &mut Command) -> &mut Command {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    cmd
}

/// Kills a process together with its children (yt-dlp spawns ffmpeg, and
/// yt-dlp.exe / yt-dlp_macos are themselves a launcher process plus a Python
/// child). On Unix the process must have been started with `own_process_group`.
pub fn kill_tree(pid: u32) {
    #[cfg(windows)]
    let _ = hidden_command("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output();
    // A negative pid addresses the whole process group.
    #[cfg(not(windows))]
    let _ = Command::new("/bin/kill")
        .args(["-TERM", "--", &format!("-{pid}")])
        .output();
}
