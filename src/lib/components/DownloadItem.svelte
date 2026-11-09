<script lang="ts">
  import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
  import {
    api,
    errorMessage,
    formatBytes,
    formatEta,
    formatPercent,
    formatSpeed,
  } from "$lib/api";
  import { belowElement, showMenu, writeClipboard, type MenuEntry } from "$lib/native.svelte";
  import { downloads, isActive, type Job } from "$lib/state/downloads.svelte";
  import Icon from "./Icon.svelte";
  import ProgressBar from "./ProgressBar.svelte";

  let { job, onerror }: { job: Job; onerror: (message: string) => void } = $props();

  let moreButton: HTMLButtonElement;
  const active = $derived(isActive(job));
  const playlistProgress = $derived(
    job.request.playlist && job.playlistIndex ? `${job.playlistIndex}/${job.playlistCount ?? "?"}` : null,
  );

  const timeFmt = new Intl.DateTimeFormat("tr-TR", { hour: "2-digit", minute: "2-digit" });
  const dayFmt = new Intl.DateTimeFormat("tr-TR", { day: "numeric", month: "short" });
  function when(ts: number | null) {
    if (!ts) return "";
    const d = new Date(ts);
    return d.toDateString() === new Date().toDateString() ? timeFmt.format(d) : dayFmt.format(d);
  }

  const status = $derived.by(() => {
    switch (job.state) {
      case "starting":
        return "Hazırlanıyor…";
      case "processing":
        return `${job.stage ?? "İşleniyor"}…`;
      case "cancelled":
        return "İptal edildi";
      case "error":
        return job.error?.split("\n")[0] ?? "Bir hata oluştu";
      case "done": {
        const parts = ["Tamamlandı"];
        if (job.doneBytes > 0) parts.push(formatBytes(job.doneBytes));
        parts.push(when(job.finishedAt));
        return parts.join(" · ");
      }
      case "downloading": {
        const parts: string[] = [];
        if (playlistProgress) parts.push(playlistProgress);
        else if (job.request.mode === "video" && job.part > 0) parts.push("Ses");
        parts.push(job.percent != null ? formatPercent(job.percent) : "İndiriliyor");
        if (job.total) parts.push(`${formatBytes(job.downloaded)} / ${formatBytes(job.total)}`);
        if (job.speed) parts.push(formatSpeed(job.speed));
        if (job.eta) parts.push(formatEta(job.eta));
        return parts.join(" · ");
      }
    }
  });

  async function open() {
    if (job.state !== "done" || !job.file) return;
    try {
      await api.openPath(job.file);
    } catch (e) {
      onerror(errorMessage(e));
    }
  }

  async function reveal() {
    if (!job.file) return;
    try {
      await revealItemInDir(job.file);
    } catch {
      onerror("Dosya bulunamadı. Taşınmış ya da silinmiş olabilir.");
    }
  }

  function entries(): MenuEntry[] {
    const list: MenuEntry[] = [];
    if (job.state === "done" && job.file) {
      list.push({ text: "Aç", action: open }, { text: "Klasörde göster", action: reveal }, "separator");
    }
    list.push(
      { text: "Bağlantıyı kopyala", action: () => writeClipboard(job.request.url) },
      { text: "Tarayıcıda aç", action: () => openUrl(job.request.url).catch(() => {}) },
    );
    if (job.state === "error" && job.error) {
      list.push({ text: "Hata ayrıntısını kopyala", action: () => writeClipboard(job.error ?? "") });
    }
    list.push("separator");
    if (active) {
      list.push({ text: "İndirmeyi iptal et", action: () => downloads.cancel(job) });
    } else {
      list.push(
        { text: job.state === "done" ? "Yeniden indir" : "Yeniden dene", action: () => downloads.retry(job) },
        { text: "Listeden kaldır", action: () => downloads.remove(job) },
      );
    }
    return list;
  }

  function oncontextmenu(e: MouseEvent) {
    e.preventDefault();
    showMenu(entries(), { x: e.clientX, y: e.clientY });
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.target !== e.currentTarget) return;
    if (e.key === "Enter") open();
    else if (e.key === "Delete" && !active) downloads.remove(job);
    else if (e.key === "ContextMenu" || (e.key === "F10" && e.shiftKey)) {
      e.preventDefault();
      const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
      showMenu(entries(), { x: r.left + 48, y: r.top + r.height / 2 });
    } else return;
  }
</script>

<!-- Rows are focusable like a native list: Enter opens, Delete removes, menu key shows the context menu. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="item {job.state}"
  role="listitem"
  tabindex="0"
  title={job.state === "error" ? (job.error ?? undefined) : undefined}
  ondblclick={open}
  {oncontextmenu}
  {onkeydown}
>
  <div class="thumb" class:audio={job.request.mode === "audio"}>
    {#if job.request.mode === "audio"}
      <Icon name="audio" size={20} />
    {:else if job.thumbnail}
      <img src={job.thumbnail} alt="" referrerpolicy="no-referrer" draggable="false" />
    {:else}
      <Icon name={job.request.playlist ? "list" : "video"} size={20} />
    {/if}
    {#if job.state === "done"}
      <span class="badge ok"><Icon name="check" size={10} /></span>
    {:else if job.state === "error"}
      <span class="badge bad"><Icon name="cancel" size={8} /></span>
    {/if}
  </div>

  <div class="text">
    <div class="ellipsis">{job.title}</div>
    <div class="caption meta">
      <span class="label">{job.label}</span>
      <span class="sep">·</span>
      <span class="ellipsis status" class:critical={job.state === "error"}>{status}</span>
    </div>
    {#if playlistProgress && job.current && active}
      <div class="caption text-2 ellipsis">{job.current}</div>
    {/if}
    {#if active}
      <div class="bar">
        <ProgressBar
          value={job.state === "downloading" ? job.percent : null}
        />
      </div>
    {/if}
  </div>

  <div class="actions">
    {#if active}
      <button class="btn subtle icon" title="İptal et" aria-label="İndirmeyi iptal et" onclick={() => downloads.cancel(job)}>
        <Icon name="cancel" size={14} />
      </button>
    {:else}
      {#if job.state === "done" && job.file}
        <button class="btn subtle icon" title="Aç" aria-label="Dosyayı aç" onclick={open}>
          <Icon name="openFile" />
        </button>
        <button class="btn subtle icon" title="Klasörde göster" aria-label="Klasörde göster" onclick={reveal}>
          <Icon name="folderOpen" />
        </button>
      {:else}
        <button class="btn subtle icon" title="Yeniden dene" aria-label="Yeniden dene" onclick={() => downloads.retry(job)}>
          <Icon name="refresh" />
        </button>
      {/if}
    {/if}
    <button
      bind:this={moreButton}
      class="btn subtle icon"
      title="Diğer seçenekler"
      aria-label="Diğer seçenekler"
      onclick={() => showMenu(entries(), belowElement(moreButton))}
    >
      <Icon name="more" />
    </button>
  </div>
</div>

<style>
  .item {
    display: flex;
    align-items: center;
    gap: 14px;
    min-height: 64px;
    padding: 10px 8px 10px 12px;
    box-sizing: border-box;
    background: var(--fill-card);
    border: 1px solid var(--stroke-card);
    border-radius: var(--radius-control);
    background-clip: padding-box;
    animation: enter 0.25s var(--ease-out);
  }
  .item:hover {
    background-color: var(--fill-card-2);
  }
  .item:focus-visible {
    outline-offset: -1px;
  }
  .thumb {
    position: relative;
    width: 72px;
    height: 40px;
    flex-shrink: 0;
    border-radius: var(--radius-control);
    background: var(--fill-alt);
    color: var(--text-2);
    display: grid;
    place-items: center;
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: var(--radius-control);
  }
  .thumb.audio {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    color: var(--accent-text);
  }
  .badge {
    position: absolute;
    right: -5px;
    bottom: -5px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--text-inverse);
    box-shadow: 0 0 0 2px var(--bg-solid);
  }
  .badge.ok {
    background: var(--success);
  }
  .badge.bad {
    background: var(--critical);
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .meta {
    display: flex;
    gap: 6px;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
    min-width: 0;
  }
  .meta .label {
    flex-shrink: 0;
  }
  .meta .sep {
    color: var(--text-3);
  }
  .status.critical {
    color: var(--critical);
  }
  .bar {
    margin-top: 7px;
    margin-bottom: 2px;
  }
  .actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }
  .actions .btn {
    color: var(--text-2);
  }
  .actions .btn:hover {
    color: var(--text-1);
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
  }
</style>
