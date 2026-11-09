// yt-dlp / ffmpeg availability and installation progress.
import { api, errorMessage, onToolProgress, type ToolStatus } from "$lib/api";

export type ToolTask = "yt-dlp" | "ffmpeg" | "update";

class Tools {
  status = $state<ToolStatus | null>(null);
  busy = $state<ToolTask | null>(null);
  bytes = $state(0);
  phase = $state<"download" | "extract">("download");
  checking = $state(true);

  ready = $derived(!!this.status?.ytdlpVersion);

  /** Needs the Tauri backend, so it runs after startup rather than on import. */
  async init() {
    onToolProgress((p) => {
      this.bytes = p.bytes;
      this.phase = p.phase;
    });
    await this.refresh();
  }

  async refresh() {
    this.checking = true;
    try {
      this.status = await api.toolStatus();
    } catch (e) {
      // Fall through to the setup screen instead of a blank window.
      this.status ??= { ytdlpVersion: null, ffmpeg: false, binDir: "" };
      throw e;
    } finally {
      this.checking = false;
    }
  }

  /** Throws a user-facing message on failure. */
  async install(kind: "yt-dlp" | "ffmpeg") {
    this.busy = kind;
    this.bytes = 0;
    this.phase = "download";
    try {
      this.status = kind === "yt-dlp" ? await api.installYtdlp() : await api.installFfmpeg();
    } catch (e) {
      throw errorMessage(e);
    } finally {
      this.busy = null;
    }
  }

  /** Returns yt-dlp's last output line, e.g. "yt-dlp is up to date (…)". */
  async update(): Promise<string> {
    this.busy = "update";
    try {
      const out = await api.updateYtdlp();
      const before = this.status?.ytdlpVersion;
      await this.refresh();
      const after = this.status?.ytdlpVersion;
      if (after && before && after !== before) return `yt-dlp ${after} sürümüne güncellendi.`;
      return after ? `yt-dlp güncel (${after}).` : (out.split("\n").filter(Boolean).pop() ?? "");
    } catch (e) {
      throw errorMessage(e);
    } finally {
      this.busy = null;
    }
  }
}

export const tools = new Tools();
