// User preferences, remembered between launches.
import { downloadDir } from "@tauri-apps/api/path";

const KEY = "settings.v1";

type Saved = {
  outDir: string;
  mode: "video" | "audio";
  audioFormat: "mp3" | "m4a";
  preferredHeight: number | null;
  downloadFfmpeg: boolean;
};

function load(): Partial<Saved> {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? "{}");
  } catch {
    return {};
  }
}

class Settings {
  outDir = $state("");
  mode = $state<"video" | "audio">("video");
  audioFormat = $state<"mp3" | "m4a">("mp3");
  /** null = best available */
  preferredHeight = $state<number | null>(null);
  downloadFfmpeg = $state(true);

  constructor() {
    const s = load();
    this.outDir = s.outDir ?? "";
    this.mode = s.mode ?? "video";
    this.audioFormat = s.audioFormat ?? "mp3";
    this.preferredHeight = s.preferredHeight ?? null;
    this.downloadFfmpeg = s.downloadFfmpeg ?? true;

    $effect.root(() => {
      $effect(() => {
        const data: Saved = {
          outDir: this.outDir,
          mode: this.mode,
          audioFormat: this.audioFormat,
          preferredHeight: this.preferredHeight,
          downloadFfmpeg: this.downloadFfmpeg,
        };
        try {
          localStorage.setItem(KEY, JSON.stringify(data));
        } catch {}
      });
    });
  }

  /** Needs the Tauri backend, so it runs after startup rather than on import. */
  async init() {
    if (!this.outDir) this.outDir = await downloadDir().catch(() => "");
  }
}

export const settings = new Settings();
