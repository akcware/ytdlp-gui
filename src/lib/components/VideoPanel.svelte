<!-- The fetched video: preview card + Settings-style download options. -->
<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { formatBytes, formatDuration, hostOf, type VideoInfo } from "$lib/api";
  import { t, tp } from "$lib/i18n.svelte";
  import { shortcut } from "$lib/platform";
  import { downloads } from "$lib/state/downloads.svelte";
  import { settings } from "$lib/state/settings.svelte";
  import { tools } from "$lib/state/tools.svelte";
  import ComboBox from "./ComboBox.svelte";
  import Icon from "./Icon.svelte";
  import Segmented from "./Segmented.svelte";

  let { info, url, onclose }: { info: VideoInfo; url: string; onclose: () => void } = $props();

  const GENERIC_HEIGHTS = [2160, 1440, 1080, 720, 480, 360];
  const ffmpeg = $derived(!!tools.status?.ffmpeg);

  const qualityItems = $derived.by(() => {
    const qs =
      !info.isPlaylist && info.qualities.length
        ? info.qualities
        : GENERIC_HEIGHTS.map((height) => ({ height, fps: null, approxBytes: null }));
    return [
      {
        value: null as number | null,
        label: t("bestQuality"),
        detail: qs[0]?.approxBytes ? `~${formatBytes(qs[0].approxBytes)}` : undefined,
      },
      ...qs.map((q) => ({
        value: q.height as number | null,
        label: `${q.height}p${q.fps && q.fps > 30 ? Math.round(q.fps) : ""}${q.height >= 2160 ? " 4K" : q.height >= 1440 ? " 2K" : ""}`,
        detail: q.approxBytes ? `~${formatBytes(q.approxBytes)}` : undefined,
      })),
    ];
  });

  // Keep the remembered quality if this video has it, otherwise "best".
  const quality = $derived(
    qualityItems.some((i) => i.value === settings.preferredHeight) ? settings.preferredHeight : null,
  );

  const audioItems = $derived([
    { value: "mp3" as const, label: "MP3", detail: t("mp3Detail") },
    { value: "m4a" as const, label: "M4A (AAC)", detail: t("m4aDetail") },
  ]);

  const qualityHint = $derived(
    !ffmpeg ? t("qualityHintNoFfmpeg") : info.isPlaylist ? t("qualityHintPlaylist") : t("qualityHint"),
  );

  async function browse() {
    const dir = await open({ directory: true, defaultPath: settings.outDir || undefined, title: t("chooseFolder") });
    if (typeof dir === "string") settings.outDir = dir;
  }

  export async function download() {
    if (!settings.outDir) return browse();
    const mode = settings.mode;
    // Language-neutral label; an empty one is shown as "Best quality".
    const label = mode === "audio" ? (ffmpeg ? settings.audioFormat.toUpperCase() : "M4A") : quality ? `${quality}p` : "";
    await downloads.start(
      {
        url,
        mode,
        maxHeight: mode === "video" ? quality : null,
        audioFormat: mode === "audio" ? settings.audioFormat : null,
        outDir: settings.outDir,
        playlist: info.isPlaylist,
      },
      { title: info.title, thumbnail: info.thumbnail, label },
    );
  }
</script>

<section class="card preview">
  <div class="thumb">
    {#if info.thumbnail}
      <img src={info.thumbnail} alt="" referrerpolicy="no-referrer" draggable="false" />
    {:else}
      <Icon name={info.isPlaylist ? "list" : "video"} size={32} />
    {/if}
    {#if info.duration}<span class="duration caption">{formatDuration(info.duration)}</span>{/if}
  </div>
  <div class="meta">
    <h2 class="subtitle" title={info.title}>{info.title || t("untitled")}</h2>
    <div class="caption text-2 line">
      {#if info.isPlaylist}
        <span>{info.entryCount != null ? tp("playlistCount", info.entryCount) : "Playlist"}</span>
      {:else if info.uploader}
        <span class="ellipsis">{info.uploader}</span>
      {/if}
      {#if hostOf(url)}<span class="host"><Icon name="globe" size={12} />{hostOf(url)}</span>{/if}
    </div>
  </div>
  <button class="btn subtle icon close" title={t("closeVideo")} aria-label={t("closeVideoLabel")} onclick={onclose}>
    <Icon name="cancel" size={12} />
  </button>
</section>

<div class="group-header">{t("downloadOptions")}</div>
<div class="setting-group">
  <div class="setting">
    <Icon name={settings.mode === "video" ? "video" : "audio"} size={20} />
    <div class="setting-text">
      <span>{t("type")}</span>
      <span class="caption">{t("typeHint")}</span>
    </div>
    <Segmented
      label={t("downloadType")}
      bind:value={settings.mode}
      items={[
        { value: "video", label: t("video"), icon: "video" },
        { value: "audio", label: t("audio"), icon: "audio" },
      ]}
    />
  </div>

  {#if settings.mode === "video"}
    <div class="setting">
      <Icon name="settings" size={20} />
      <div class="setting-text">
        <span>{t("quality")}</span>
        <span class="caption">{qualityHint}</span>
      </div>
      <ComboBox
        label={t("quality")}
        items={qualityItems}
        bind:value={() => quality, (v) => (settings.preferredHeight = v)}
      />
    </div>
  {:else}
    <div class="setting">
      <Icon name="audio" size={20} />
      <div class="setting-text">
        <span>{t("audioFormat")}</span>
        <span class="caption">{ffmpeg ? t("audioHint") : t("audioHintNoFfmpeg")}</span>
      </div>
      <ComboBox label={t("audioFormat")} items={audioItems} bind:value={settings.audioFormat} disabled={!ffmpeg} />
    </div>
  {/if}

  <div class="setting">
    <Icon name="folder" size={20} />
    <div class="setting-text">
      <span>{t("saveLocation")}</span>
      <span class="caption ellipsis" title={settings.outDir}>{settings.outDir || t("notSelected")}</span>
    </div>
    <button class="btn" onclick={browse}>{t("browse")}</button>
  </div>
</div>

<div class="actions">
  <span class="caption text-3">{shortcut("Enter")}</span>
  <button class="btn accent" onclick={download}>
    <Icon name="download" />
    {#if !info.isPlaylist}
      {t("download")}
    {:else if info.entryCount != null}
      {tp("downloadVideos", info.entryCount)}
    {:else}
      {t("downloadPlaylist")}
    {/if}
  </button>
</div>

<style>
  .preview {
    position: relative;
    display: flex;
    gap: 16px;
    padding: 12px;
  }
  .thumb {
    position: relative;
    width: 208px;
    aspect-ratio: 16 / 9;
    flex-shrink: 0;
    border-radius: var(--radius-control);
    overflow: hidden;
    background: var(--fill-alt);
    display: grid;
    place-items: center;
    color: var(--text-2);
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .duration {
    position: absolute;
    right: 6px;
    bottom: 6px;
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(0, 0, 0, 0.72);
    color: #fff;
    font-variant-numeric: tabular-nums;
  }
  .meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 4px 36px 4px 0;
  }
  h2 {
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .host {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }
  .close {
    position: absolute;
    top: 8px;
    right: 8px;
    color: var(--text-2);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 12px;
    margin-top: 12px;
  }
  .actions .btn {
    min-width: 120px;
  }
  @media (max-width: 640px) {
    .thumb {
      width: 136px;
    }
    .subtitle {
      font-size: 16px;
      line-height: 22px;
    }
  }

  :global(.macos) .preview {
    padding: 10px;
    gap: 14px;
  }
  :global(.macos) .thumb {
    border-radius: 6px;
  }
  :global(.macos) .duration {
    border-radius: 4px;
    font-weight: 500;
  }
  :global(.macos) .close {
    top: 6px;
    right: 6px;
  }
  :global(.macos) .actions {
    margin-top: 14px;
    gap: 10px;
  }
  :global(.macos) .actions .btn {
    min-width: 110px;
    min-height: 28px;
  }
</style>
