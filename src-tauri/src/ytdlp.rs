//! Talks to yt-dlp through its command line, using its machine-readable
//! outputs: `-J` for metadata, `--progress-template` for progress and
//! `--print after_move:` for the final file path.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use std::process::Stdio;
use std::sync::{Mutex, mpsc};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::process::{blocking, hidden_command, kill_tree};
use crate::tools::{ffmpeg_available, local_ffmpeg, ytdlp_path};

// ---------------------------------------------------------------------------
// Metadata (-J)
// ---------------------------------------------------------------------------

/// The subset of yt-dlp's `-J` output we use. Numbers are read as f64
/// because extractors are not consistent about ints vs floats.
#[derive(Deserialize)]
struct RawInfo {
    #[serde(rename = "_type")]
    kind: Option<String>,
    id: Option<String>,
    title: Option<String>,
    uploader: Option<String>,
    channel: Option<String>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    formats: Option<Vec<RawFormat>>,
    entries: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize)]
struct RawFormat {
    ext: Option<String>,
    height: Option<f64>,
    fps: Option<f64>,
    vcodec: Option<String>,
    acodec: Option<String>,
    filesize: Option<f64>,
    filesize_approx: Option<f64>,
    abr: Option<f64>,
}

impl RawFormat {
    fn has_video(&self) -> bool {
        !matches!(self.vcodec.as_deref(), None | Some("none"))
            && self.ext.as_deref() != Some("mhtml") // storyboards
    }
    fn is_audio_only(&self) -> bool {
        !self.has_video() && !matches!(self.acodec.as_deref(), None | Some("none"))
    }
    fn size(&self) -> Option<f64> {
        self.filesize.or(self.filesize_approx)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quality {
    height: u32,
    fps: Option<f64>,
    /// Rough size of best video at this height + best audio.
    approx_bytes: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    id: String,
    title: String,
    uploader: Option<String>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    is_playlist: bool,
    entry_count: Option<usize>,
    /// Available video heights, highest first.
    qualities: Vec<Quality>,
}

fn summarize(raw: RawInfo) -> VideoInfo {
    let formats = raw.formats.unwrap_or_default();

    let best_audio = formats
        .iter()
        .filter(|f| f.is_audio_only())
        .max_by(|a, b| a.abr.unwrap_or(0.0).total_cmp(&b.abr.unwrap_or(0.0)))
        .and_then(RawFormat::size);

    let mut by_height: BTreeMap<u32, (Option<f64>, Option<f64>)> = BTreeMap::new();
    for f in formats.iter().filter(|f| f.has_video()) {
        let Some(h) = f.height.filter(|h| *h > 0.0) else {
            continue;
        };
        let entry = by_height.entry(h as u32).or_insert((None, None));
        entry.0 = max_opt(entry.0, f.fps);
        entry.1 = max_opt(entry.1, f.size());
    }

    let qualities = by_height
        .into_iter()
        .rev()
        .map(|(height, (fps, video_size))| Quality {
            height,
            fps,
            approx_bytes: video_size.map(|v| (v + best_audio.unwrap_or(0.0)) as u64),
        })
        .collect();

    VideoInfo {
        id: raw.id.unwrap_or_default(),
        title: raw.title.unwrap_or_default(), // the UI shows "(untitled)"
        uploader: raw.uploader.or(raw.channel),
        duration: raw.duration,
        thumbnail: raw.thumbnail,
        is_playlist: raw.kind.as_deref() == Some("playlist"),
        entry_count: raw.entries.map(|e| e.len()),
        qualities,
    }
}

fn max_opt(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    }
}

/// Picks the most useful lines out of yt-dlp's stderr for an error message.
fn error_text(stderr: &str) -> String {
    let errors: Vec<&str> = stderr
        .lines()
        .filter(|l| l.starts_with("ERROR:"))
        .collect();
    if errors.is_empty() {
        stderr.trim().lines().last().unwrap_or("Unknown error").to_string()
    } else {
        errors.join("\n")
    }
}

#[tauri::command]
pub async fn fetch_info(app: AppHandle, url: String) -> Result<VideoInfo, String> {
    blocking(move || {
        let out = hidden_command(ytdlp_path(&app)?)
            .args([
                "-J",
                "--no-playlist",
                "--flat-playlist", // playlists: list entries only, much faster
                "--no-warnings",
                "--encoding",
                "utf-8",
                "--",
            ])
            .arg(&url)
            .output()
            .map_err(|e| format!("Couldn't run yt-dlp: {e}"))?;
        if !out.status.success() {
            return Err(error_text(&String::from_utf8_lossy(&out.stderr)));
        }
        let raw: RawInfo = serde_json::from_slice(&out.stdout)
            .map_err(|e| format!("Couldn't read yt-dlp's output: {e}"))?;
        Ok(summarize(raw))
    })
    .await
}

// ---------------------------------------------------------------------------
// Downloads
// ---------------------------------------------------------------------------

/// Running downloads: job id -> process id, so they can be cancelled.
#[derive(Default)]
pub struct Jobs {
    next_id: AtomicU32,
    pids: Mutex<HashMap<u32, u32>>,
    cancelled: Mutex<HashSet<u32>>,
}

impl Jobs {
    pub fn kill_all(&self) {
        for pid in self.pids.lock().unwrap().values() {
            kill_tree(*pid);
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    url: String,
    /// "video" or "audio"
    mode: String,
    /// Video only: highest resolution wanted, `None` = best available.
    max_height: Option<u32>,
    /// Audio only: "mp3" or "m4a".
    audio_format: Option<String>,
    out_dir: String,
    playlist: bool,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
enum DownloadEvent {
    Progress {
        id: u32,
        status: String,
        percent: Option<f64>,
        downloaded: Option<f64>,
        total: Option<f64>,
        speed: Option<f64>,
        eta: Option<f64>,
        playlist_index: Option<u32>,
        playlist_count: Option<u32>,
        title: Option<String>,
    },
    Postprocess {
        id: u32,
        status: String,
        processor: String,
    },
    File {
        id: u32,
        path: String,
    },
    Done {
        id: u32,
        ok: bool,
        cancelled: bool,
        error: Option<String>,
    },
}

// Line markers for our own templates on stdout. Missing fields print "NA".
const P: &str = "@@P|";
const PP: &str = "@@PP|";
const F: &str = "@@F|";

fn progress_template() -> String {
    // Title goes last because it may itself contain '|'.
    format!(
        "download:{P}%(progress.status)s|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|\
         %(progress.total_bytes_estimate)s|%(progress.speed)s|%(progress.eta)s|\
         %(progress.fragment_index)s|%(progress.fragment_count)s|\
         %(info.playlist_index)s|%(info.n_entries)s|%(info.title)s"
    )
}

fn push(args: &mut Vec<String>, items: &[&str]) {
    args.extend(items.iter().map(|s| s.to_string()));
}

fn build_args(req: &DownloadRequest, has_ffmpeg: bool) -> Result<Vec<String>, String> {
    let mut args: Vec<String> = Vec::new();

    match req.mode.as_str() {
        "video" => {
            if has_ffmpeg {
                // Best video + best audio, merged into an mp4.
                push(&mut args, &["-f", "bv*+ba/b", "--merge-output-format", "mp4"]);
            } else {
                // Without ffmpeg only single-file formats work (usually ≤360p on YouTube).
                push(&mut args, &["-f", "b"]);
            }
            if let Some(h) = req.max_height {
                // Largest resolution not above h (falls back gracefully).
                push(&mut args, &["-S", &format!("res:{h}")]);
            }
        }
        "audio" => {
            let fmt = req.audio_format.as_deref().unwrap_or("mp3");
            if !matches!(fmt, "mp3" | "m4a") {
                return Err(format!("Unsupported audio format: {fmt}"));
            }
            if has_ffmpeg {
                push(
                    &mut args,
                    &["-f", "ba/b", "-x", "--audio-format", fmt, "--audio-quality", "0"],
                );
            } else {
                // No conversion possible: take the original m4a stream.
                push(&mut args, &["-f", "ba[ext=m4a]/ba"]);
            }
        }
        other => return Err(format!("Unknown mode: {other}")),
    }

    let template = if req.playlist {
        "%(playlist_title)s/%(playlist_index)03d - %(title)s [%(id)s].%(ext)s"
    } else {
        "%(title)s [%(id)s].%(ext)s"
    };
    let progress = progress_template();
    let postprocess = format!("postprocess:{PP}%(progress.status)s|%(progress.postprocessor)s");
    let print_file = format!("after_move:{F}%(filepath)s");

    push(&mut args, &[
        "--newline",
        "--progress", // --print implies --quiet; keep progress lines anyway
        "--encoding",
        "utf-8",
        "--no-playlist", // watch?v=..&list=..: only the video
        "-P",
        &req.out_dir,
        "-o",
        template,
        "--progress-template",
        &progress,
        "--progress-template",
        &postprocess,
        "--print",
        &print_file,
    ]);
    Ok(args)
}

fn num(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|n| n.is_finite())
}

fn parse_progress(id: u32, line: &str) -> Option<DownloadEvent> {
    let parts: Vec<&str> = line.splitn(11, '|').collect();
    if parts.len() < 11 {
        return None;
    }
    let downloaded = num(parts[1]);
    let total = num(parts[2]).or(num(parts[3]));
    let percent = match (downloaded, total, num(parts[6]), num(parts[7])) {
        (Some(d), Some(t), _, _) if t > 0.0 => Some(d / t * 100.0),
        (_, _, Some(i), Some(n)) if n > 0.0 => Some(i / n * 100.0),
        _ => None,
    };
    Some(DownloadEvent::Progress {
        id,
        status: parts[0].to_string(),
        percent: percent.map(|p| p.clamp(0.0, 100.0)),
        downloaded,
        total,
        speed: num(parts[4]),
        eta: num(parts[5]),
        playlist_index: num(parts[8]).map(|n| n as u32),
        playlist_count: num(parts[9]).map(|n| n as u32),
        title: Some(parts[10].to_string()).filter(|t| t != "NA"),
    })
}

/// Reads `reader` line by line on a new thread and sends `(is_stderr, line)`.
fn pipe_lines(reader: impl Read + Send + 'static, is_stderr: bool, tx: mpsc::Sender<(bool, String)>) {
    thread::spawn(move || {
        for line in BufReader::new(reader).split(b'\n').map_while(Result::ok) {
            let text = String::from_utf8_lossy(&line).trim_end().to_string();
            if tx.send((is_stderr, text)).is_err() {
                break;
            }
        }
    });
}

#[tauri::command]
pub fn start_download(
    app: AppHandle,
    jobs: State<'_, Jobs>,
    request: DownloadRequest,
) -> Result<u32, String> {
    let mut args = build_args(&request, ffmpeg_available(&app))?;
    if let Some(ffmpeg) = local_ffmpeg(&app) {
        args.push("--ffmpeg-location".into());
        args.push(ffmpeg.to_string_lossy().into_owned());
    }
    args.push("--".into());
    args.push(request.url.clone());

    let mut child = hidden_command(ytdlp_path(&app)?)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Couldn't start yt-dlp: {e}"))?;

    let id = jobs.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    jobs.pids.lock().unwrap().insert(id, child.id());

    // Both pipes are drained on their own threads (a full pipe would block
    // yt-dlp) and funnelled into one channel. Markers are recognised on either
    // stream, so it doesn't matter where yt-dlp decides to print progress.
    let (tx, rx) = mpsc::channel::<(bool, String)>();
    pipe_lines(child.stdout.take().expect("stdout is piped"), false, tx.clone());
    pipe_lines(child.stderr.take().expect("stderr is piped"), true, tx);

    thread::spawn(move || {
        let emit = |event: DownloadEvent| {
            let _ = app.emit("download-event", event);
        };
        let mut last_progress: Option<Instant> = None;
        let mut stderr_text = String::new();

        // Ends when both pipes are closed, i.e. yt-dlp has exited.
        for (is_stderr, line) in rx {
            if let Some(rest) = line.strip_prefix(PP) {
                let (status, processor) = rest.split_once('|').unwrap_or((rest, ""));
                emit(DownloadEvent::Postprocess {
                    id,
                    status: status.to_string(),
                    processor: processor.to_string(),
                });
            } else if let Some(rest) = line.strip_prefix(P) {
                // Throttle to ~8 updates/s, but never drop state changes.
                let is_update = rest.starts_with("downloading|");
                let too_soon =
                    last_progress.is_some_and(|t| t.elapsed() < Duration::from_millis(120));
                if is_update && too_soon {
                    continue;
                }
                last_progress = Some(Instant::now());
                if let Some(event) = parse_progress(id, rest) {
                    emit(event);
                }
            } else if let Some(path) = line.strip_prefix(F) {
                emit(DownloadEvent::File {
                    id,
                    path: path.to_string(),
                });
            } else if is_stderr {
                stderr_text.push_str(&line);
                stderr_text.push('\n');
            }
        }

        let ok = child.wait().map(|s| s.success()).unwrap_or(false);

        let jobs = app.state::<Jobs>();
        jobs.pids.lock().unwrap().remove(&id);
        let cancelled = jobs.cancelled.lock().unwrap().remove(&id);

        emit(DownloadEvent::Done {
            id,
            ok,
            cancelled,
            error: (!ok && !cancelled).then(|| error_text(&stderr_text)),
        });
    });

    Ok(id)
}

#[tauri::command]
pub fn cancel_download(jobs: State<'_, Jobs>, id: u32) {
    let pid = jobs.pids.lock().unwrap().get(&id).copied();
    if let Some(pid) = pid {
        jobs.cancelled.lock().unwrap().insert(id);
        kill_tree(pid);
    }
}
