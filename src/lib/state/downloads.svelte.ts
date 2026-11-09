// Download jobs: starting/cancelling yt-dlp runs, applying their progress
// events, and keeping finished items as history between launches.
import {
  api,
  errorMessage,
  onDownloadEvent,
  type DownloadEvent,
  type DownloadRequest,
} from "$lib/api";

export type JobState = "starting" | "downloading" | "processing" | "done" | "error" | "cancelled";

export type Job = {
  key: string;
  backendId: number | null;
  request: DownloadRequest;
  title: string;
  thumbnail: string | null;
  /** "1080p", "MP3", "En iyi kalite"… */
  label: string;
  state: JobState;
  percent: number | null;
  downloaded: number | null;
  total: number | null;
  speed: number | null;
  eta: number | null;
  stage: string | null;
  /** finished streams (video + audio = 2) */
  part: number;
  /** bytes of streams already finished */
  doneBytes: number;
  /** playlist: title of the item being downloaded */
  current: string | null;
  playlistIndex: number | null;
  playlistCount: number | null;
  file: string | null;
  error: string | null;
  finishedAt: number | null;
};

const HISTORY_KEY = "history.v1";
const HISTORY_LIMIT = 100;

const STAGES: Record<string, string> = {
  Merger: "Video ve ses birleştiriliyor",
  ExtractAudio: "Ses dönüştürülüyor",
  VideoConvertor: "Video dönüştürülüyor",
  MoveFiles: "Dosya taşınıyor",
};

export const isActive = (j: Job) =>
  j.state === "starting" || j.state === "downloading" || j.state === "processing";

const newKey = () =>
  globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(36).slice(2)}`;

function loadHistory(): Job[] {
  try {
    const saved = JSON.parse(localStorage.getItem(HISTORY_KEY) ?? "[]");
    return Array.isArray(saved) ? saved.filter((j) => j && !isActive(j)) : [];
  } catch {
    return [];
  }
}

class Downloads {
  jobs = $state<Job[]>(loadHistory());
  active = $derived(this.jobs.filter(isActive));
  finished = $derived(this.jobs.filter((j) => !isActive(j)));

  /** Called when a job ends (done or failed), e.g. to show a toast. */
  onFinished: ((job: Job) => void) | null = null;

  #byBackend = new Map<number, string>();
  #early = new Map<number, DownloadEvent[]>();

  /** Needs the Tauri backend, so it runs after startup rather than on import. */
  init() {
    onDownloadEvent((e) => this.#handle(e));
  }

  async start(request: DownloadRequest, meta: { title: string; thumbnail: string | null; label: string }) {
    this.jobs.unshift({
      key: newKey(),
      backendId: null,
      request,
      title: meta.title,
      thumbnail: meta.thumbnail,
      label: meta.label,
      ...this.#fresh(),
    });
    await this.#launch(this.jobs[0]);
  }

  async retry(job: Job) {
    Object.assign(job, this.#fresh());
    // Move it to the top, like a new download.
    this.jobs = [job, ...this.jobs.filter((j) => j.key !== job.key)];
    await this.#launch(this.jobs[0]);
  }

  cancel(job: Job) {
    if (job.backendId != null && isActive(job)) api.cancelDownload(job.backendId);
  }

  remove(job: Job) {
    if (isActive(job)) return;
    this.jobs = this.jobs.filter((j) => j.key !== job.key);
    this.#persist();
  }

  clearFinished() {
    this.jobs = this.jobs.filter(isActive);
    this.#persist();
  }

  #fresh() {
    return {
      state: "starting" as JobState,
      percent: null,
      downloaded: null,
      total: null,
      speed: null,
      eta: null,
      stage: null,
      part: 0,
      doneBytes: 0,
      current: null,
      playlistIndex: null,
      playlistCount: null,
      file: null,
      error: null,
      finishedAt: null,
    };
  }

  async #launch(job: Job) {
    try {
      const id = await api.startDownload($state.snapshot(job.request));
      job.backendId = id;
      this.#byBackend.set(id, job.key);
      // Events can arrive before start_download returns.
      for (const e of this.#early.get(id) ?? []) this.#apply(job, e);
      this.#early.delete(id);
    } catch (e) {
      job.state = "error";
      job.error = errorMessage(e);
      job.finishedAt = Date.now();
      this.#persist();
      this.onFinished?.(job);
    }
  }

  #handle(e: DownloadEvent) {
    const key = this.#byBackend.get(e.id);
    const job = key ? this.jobs.find((j) => j.key === key) : undefined;
    if (job) {
      this.#apply(job, e);
    } else {
      const list = this.#early.get(e.id) ?? [];
      list.push(e);
      this.#early.set(e.id, list);
    }
  }

  #apply(job: Job, e: DownloadEvent) {
    switch (e.kind) {
      case "progress":
        job.current = e.title;
        job.playlistIndex = e.playlistIndex;
        job.playlistCount = e.playlistCount;
        if (e.status === "downloading") {
          job.state = "downloading";
          job.stage = null;
          job.percent = e.percent;
          job.downloaded = e.downloaded;
          job.total = e.total;
          job.speed = e.speed;
          job.eta = e.eta;
        } else if (e.status === "finished") {
          job.percent = 100;
          job.part += 1;
          job.doneBytes += e.total ?? e.downloaded ?? 0;
          job.speed = null;
          job.eta = null;
        }
        break;
      case "postprocess":
        if (e.status === "started") {
          job.state = "processing";
          job.stage = STAGES[e.processor] ?? "İşleniyor";
        }
        break;
      case "file":
        job.file = e.path;
        break;
      case "done":
        job.state = e.ok ? "done" : e.cancelled ? "cancelled" : "error";
        job.error = e.error ? errorMessage(e.error) : null;
        job.speed = null;
        job.eta = null;
        job.finishedAt = Date.now();
        if (e.ok) job.percent = 100;
        if (job.backendId != null) this.#byBackend.delete(job.backendId);
        this.#persist();
        this.onFinished?.(job);
        break;
    }
  }

  #persist() {
    try {
      const finished = $state.snapshot(this.jobs).filter((j) => !isActive(j)).slice(0, HISTORY_LIMIT);
      localStorage.setItem(HISTORY_KEY, JSON.stringify(finished));
    } catch {}
  }
}

export const downloads = new Downloads();
