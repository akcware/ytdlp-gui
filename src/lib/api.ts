// Typed wrappers around the Rust commands in src-tauri/src/*.rs.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { i18n, t } from "./i18n.svelte";

export type Appearance = {
  backdrop: "mica" | "none";
  accent: {
    light3: string;
    light2: string;
    light1: string;
    base: string;
    dark1: string;
    dark2: string;
    dark3: string;
  };
};

export type ToolStatus = {
  ytdlpVersion: string | null;
  ffmpeg: boolean;
  binDir: string;
};

export type Quality = {
  height: number;
  fps: number | null;
  approxBytes: number | null;
};

export type VideoInfo = {
  id: string;
  title: string;
  uploader: string | null;
  duration: number | null;
  thumbnail: string | null;
  isPlaylist: boolean;
  entryCount: number | null;
  qualities: Quality[];
};

export type DownloadRequest = {
  url: string;
  mode: "video" | "audio";
  maxHeight: number | null;
  audioFormat: "mp3" | "m4a" | null;
  outDir: string;
  playlist: boolean;
};

export type DownloadEvent =
  | {
      kind: "progress";
      id: number;
      status: string; // "downloading" | "finished" | "error"
      percent: number | null;
      downloaded: number | null;
      total: number | null;
      speed: number | null;
      eta: number | null;
      playlistIndex: number | null;
      playlistCount: number | null;
      title: string | null;
    }
  | { kind: "postprocess"; id: number; status: string; processor: string }
  | { kind: "file"; id: number; path: string }
  | { kind: "done"; id: number; ok: boolean; cancelled: boolean; error: string | null };

export type ToolProgress = { tool: string; phase: "download" | "extract"; bytes: number };

export const api = {
  appearance: () => invoke<Appearance>("appearance"),
  toolStatus: () => invoke<ToolStatus>("tool_status"),
  installYtdlp: () => invoke<ToolStatus>("install_ytdlp"),
  updateYtdlp: () => invoke<string>("update_ytdlp"),
  installFfmpeg: () => invoke<ToolStatus>("install_ffmpeg"),
  fetchInfo: (url: string) => invoke<VideoInfo>("fetch_info", { url }),
  startDownload: (request: DownloadRequest) => invoke<number>("start_download", { request }),
  cancelDownload: (id: number) => invoke<void>("cancel_download", { id }),
  /** Opens a file with its default app, or a folder in Explorer. */
  openPath: (path: string) => invoke<void>("open_path", { path }),
};

export const onDownloadEvent = (cb: (e: DownloadEvent) => void): Promise<UnlistenFn> =>
  listen<DownloadEvent>("download-event", (e) => cb(e.payload));

export const onToolProgress = (cb: (e: ToolProgress) => void): Promise<UnlistenFn> =>
  listen<ToolProgress>("tool-progress", (e) => cb(e.payload));

// ---- formatting (follows the app language) -----------------------------------

const numberFormats = new Map<string, Intl.NumberFormat>();
function num(n: number, digits: 0 | 1): string {
  const key = `${i18n.tag}/${digits}`;
  let f = numberFormats.get(key);
  if (!f) {
    f = new Intl.NumberFormat(i18n.tag, { maximumFractionDigits: digits, minimumFractionDigits: digits });
    numberFormats.set(key, f);
  }
  return f.format(n);
}

export function formatBytes(n: number | null | undefined): string {
  if (n == null || !isFinite(n)) return "?";
  const units = [t("bytes"), "KB", "MB", "GB", "TB"];
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${num(n, i >= 2 ? 1 : 0)} ${units[i]}`;
}

export function formatSpeed(bytesPerSec: number | null | undefined): string {
  return bytesPerSec ? t("perSecond", { size: formatBytes(bytesPerSec) }) : "";
}

export function formatPercent(p: number): string {
  return t("percent", { n: num(Math.floor(p), 0) });
}

/** Remaining time, phrased like the Windows copy dialog. */
export function formatEta(sec: number | null | undefined): string {
  if (sec == null || !isFinite(sec) || sec < 0) return "";
  if (sec < 60) return t("etaSeconds", { n: Math.max(1, Math.round(sec)) });
  if (sec < 3600) return t("etaMinutes", { n: Math.ceil(sec / 60) });
  const h = Math.floor(sec / 3600);
  const m = Math.round((sec % 3600) / 60);
  return m ? t("etaHoursMinutes", { h, m }) : t("etaHours", { h });
}

export function formatDuration(sec: number | null | undefined): string {
  if (sec == null || !isFinite(sec)) return "";
  sec = Math.round(sec);
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  const mm = h ? String(m).padStart(2, "0") : String(m);
  return `${h ? h + ":" : ""}${mm}:${String(s).padStart(2, "0")}`;
}

export function hostOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return "";
  }
}

export function looksLikeUrl(text: string): boolean {
  return /^https?:\/\/\S+\.\S+/i.test(text.trim());
}

/** Turns invoke() rejections (plain strings from Rust) into a message. */
export function errorMessage(e: unknown): string {
  const text = typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
  return text.replace(/^ERROR:\s*/gm, "");
}
