// Fake backend for working on the UI in a plain browser (`bun run dev`,
// then open http://localhost:1420). Only loaded in dev mode when not running
// inside Tauri; it is never part of a production build.
//
// Try URLs containing "fail" or "playlist" to see the error and playlist paths.
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import type { DownloadEvent, ToolStatus, VideoInfo } from "./api";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const THUMB =
  "data:image/svg+xml," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 180"><defs><linearGradient id="g" x1="0" x2="1" y1="0" y2="1"><stop offset="0" stop-color="#0f3b5f"/><stop offset=".55" stop-color="#2d6a8e"/><stop offset="1" stop-color="#f2a65a"/></linearGradient></defs><rect width="320" height="180" fill="url(#g)"/><circle cx="250" cy="52" r="22" fill="#ffd9a0" opacity=".9"/><path d="M0 140 L70 95 L120 125 L190 70 L260 120 L320 90 V180 H0Z" fill="#0b2338" opacity=".85"/></svg>`,
  );

export function installMocks() {
  (globalThis as any).__YTDLP_MOCK__ = true;
  const params = new URLSearchParams(location.search);
  // ?setup=1 starts with nothing installed, to see the first-run screen.
  let ytdlpVersion: string | null = params.has("setup") ? null : "2026.10.22";
  let ffmpeg = !params.has("setup") && !params.has("noffmpeg");
  let nextId = 0;
  const timers = new Map<number, ReturnType<typeof setInterval>>();

  const status = (): ToolStatus => ({
    ytdlpVersion,
    ffmpeg,
    binDir: "C:\\Users\\demo\\AppData\\Local\\com.kadir.ytdlpgui\\bin",
  });

  const send = (e: DownloadEvent) => emit("download-event", e);

  async function fakeToolDownload(tool: string, size: number) {
    for (let bytes = 0; bytes <= size; bytes += size / 14) {
      await emit("tool-progress", { tool, phase: "download", bytes });
      await sleep(140);
    }
    if (tool === "ffmpeg") {
      await emit("tool-progress", { tool, phase: "extract", bytes: 0 });
      await sleep(900);
    }
  }

  function simulate(id: number, url: string, mode: string, playlist: boolean) {
    const fail = url.includes("fail");
    const items = playlist ? 3 : 1;
    const parts = mode === "video" ? [180e6, 9e6] : [9e6];
    let item = 1;
    let part = 0;
    let done = 0;
    const timer = setInterval(() => {
      const total = parts[part];
      done = Math.min(total, done + total / 30);
      const percent = (done / total) * 100;
      if (fail && part === 0 && percent > 40) {
        clearInterval(timer);
        timers.delete(id);
        send({ kind: "done", id, ok: false, cancelled: false, error: "ERROR: [youtube] abc123: Video unavailable. This video is private" });
        return;
      }
      send({
        kind: "progress",
        id,
        status: done >= total ? "finished" : "downloading",
        percent,
        downloaded: done,
        total,
        speed: 7.4e6,
        eta: (total - done) / 7.4e6,
        playlistIndex: playlist ? item : null,
        playlistCount: playlist ? items : null,
        title: playlist ? `Bölüm ${item}: Dağlarda bir gün` : "Gün batımında dağlar",
      });
      if (done >= total) {
        part++;
        done = 0;
        if (part >= parts.length) {
          part = 0;
          if (item < items) {
            item++;
            return;
          }
          clearInterval(timer);
          timers.delete(id);
          send({ kind: "postprocess", id, status: "started", processor: mode === "video" ? "Merger" : "ExtractAudio" });
          setTimeout(() => {
            send({ kind: "file", id, path: "C:\\Users\\demo\\Downloads\\Gün batımında dağlar [abc123].mp4" });
            send({ kind: "done", id, ok: true, cancelled: false, error: null });
          }, 1400);
        }
      }
    }, 120);
    timers.set(id, timer);
  }

  mockWindows("main");
  mockIPC(
    async (cmd, args) => {
      const a = (args ?? {}) as Record<string, any>;
      switch (cmd) {
        case "appearance":
          return {
            backdrop: "none",
            accent: { light3: "#99ebff", light2: "#60cdff", light1: "#00b7ff", base: "#0078d4", dark1: "#005fb8", dark2: "#003e92", dark3: "#001a68" },
          };
        case "tool_status":
          await sleep(250);
          return status();
        case "install_ytdlp":
          await fakeToolDownload("yt-dlp", 18e6);
          ytdlpVersion = "2026.10.22";
          return status();
        case "install_ffmpeg":
          await fakeToolDownload("ffmpeg", 96e6);
          ffmpeg = true;
          return status();
        case "update_ytdlp":
          await sleep(900);
          return "Latest version: stable@2026.10.22 from yt-dlp/yt-dlp\nyt-dlp is up to date (stable@2026.10.22 from yt-dlp/yt-dlp)";
        case "fetch_info": {
          await sleep(900);
          const url = String(a.url);
          if (url.includes("broken")) throw "ERROR: [generic] Unable to download webpage: HTTP Error 404: Not Found";
          const playlist = url.includes("playlist");
          return {
            id: "abc123",
            title: playlist ? "Dağlarda bir gün — tüm bölümler" : "Gün batımında dağlar — 4K doğa yürüyüşü",
            uploader: "Kuzey Rotası",
            duration: playlist ? null : 754,
            thumbnail: THUMB,
            isPlaylist: playlist,
            entryCount: playlist ? 3 : null,
            qualities: playlist
              ? []
              : [
                  { height: 2160, fps: 60, approxBytes: 1.24e9 },
                  { height: 1440, fps: 60, approxBytes: 6.1e8 },
                  { height: 1080, fps: 60, approxBytes: 2.4e8 },
                  { height: 720, fps: 30, approxBytes: 1.1e8 },
                  { height: 480, fps: 30, approxBytes: 5.2e7 },
                  { height: 360, fps: 30, approxBytes: 3.1e7 },
                ],
          } satisfies VideoInfo;
        }
        case "start_download": {
          const id = ++nextId;
          simulate(id, a.request.url, a.request.mode, a.request.playlist);
          return id;
        }
        case "cancel_download":
          clearInterval(timers.get(a.id));
          timers.delete(a.id);
          send({ kind: "done", id: a.id, ok: false, cancelled: true, error: null });
          return null;
        case "open_path":
          console.info("[mock] open", a.path);
          return null;
        case "plugin:path|resolve_directory":
          return "C:\\Users\\demo\\Downloads";
        case "plugin:dialog|open":
          return "D:\\Videolar";
        case "plugin:dialog|ask":
        case "plugin:dialog|confirm":
          return window.confirm(String(a.message));
        case "plugin:dialog|message":
          window.alert(String(a.message));
          return null;
        case "plugin:clipboard-manager|read_text":
          return "https://www.youtube.com/watch?v=abc123";
        case "plugin:app|version":
          return "0.1.0";
        case "plugin:notification|is_permission_granted":
          return true;
        default:
          if (!/^plugin:(window|opener|clipboard-manager|notification)\|/.test(cmd)) {
            console.warn("[mock] unhandled command", cmd, a);
          }
          return null;
      }
    },
    { shouldMockEvents: true },
  );
}
