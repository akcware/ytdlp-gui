<!-- First run: download yt-dlp (required) and ffmpeg (recommended). -->
<script lang="ts">
  import { formatBytes } from "$lib/api";
  import { settings } from "$lib/state/settings.svelte";
  import { tools } from "$lib/state/tools.svelte";
  import Icon from "./Icon.svelte";
  import InfoBar from "./InfoBar.svelte";
  import ProgressRing from "./ProgressRing.svelte";

  /** true while installing or waiting for "Başla" after installing. */
  let { active = $bindable(false) }: { active?: boolean } = $props();

  let error = $state<string | null>(null);
  let finished = $state(false);

  const hasYtdlp = $derived(!!tools.status?.ytdlpVersion);
  const hasFfmpeg = $derived(!!tools.status?.ffmpeg);
  const running = $derived(tools.busy === "yt-dlp" || tools.busy === "ffmpeg");

  async function install() {
    active = true;
    error = null;
    try {
      if (!hasYtdlp) await tools.install("yt-dlp");
      if (settings.downloadFfmpeg && !hasFfmpeg) await tools.install("ffmpeg");
      finished = true;
    } catch (e) {
      error = String(e);
    }
  }

  function progressText() {
    if (tools.phase === "extract") return "Arşiv açılıyor…";
    return tools.bytes > 0 ? `${formatBytes(tools.bytes)} indirildi` : "Bağlanıyor…";
  }
</script>

<div class="page">
  <div class="hero">
    <img src="/icon.png" alt="" width="64" height="64" draggable="false" />
    <h1 class="title">yt-dlp GUI'ye hoş geldiniz</h1>
    <p class="text-2">
      Video indirebilmek için birkaç bileşen gerekiyor. Bunlar GitHub'dan yalnızca bu uygulamanın
      klasörüne indirilir; sisteminize kurulum yapılmaz.
    </p>
  </div>

  <div class="setting-group">
    <div class="setting">
      <Icon name="download" size={20} />
      <div class="setting-text">
        <span>yt-dlp</span>
        <span class="caption">İndirme motoru · gerekli</span>
      </div>
      {@render toolState(hasYtdlp, tools.busy === "yt-dlp")}
    </div>

    <div class="setting">
      <Icon name="video" size={20} />
      <div class="setting-text">
        <span>ffmpeg</span>
        <span class="caption">720p üstü video ve MP3 dönüştürme için · önerilir</span>
      </div>
      {#if hasFfmpeg || tools.busy === "ffmpeg" || (finished && settings.downloadFfmpeg)}
        {@render toolState(hasFfmpeg, tools.busy === "ffmpeg")}
      {:else}
        <label class="checkbox">
          <input type="checkbox" bind:checked={settings.downloadFfmpeg} disabled={running} />
          İndir
        </label>
      {/if}
    </div>
  </div>

  {#if error}
    <InfoBar severity="error" title="Kurulum tamamlanamadı" message={error} onclose={() => (error = null)} />
  {/if}

  <div class="footer">
    <span class="caption text-3">Kaynak: github.com/yt-dlp</span>
    {#if finished && !error}
      <button class="btn accent wide" onclick={() => (active = false)}>Başla</button>
    {:else}
      <button class="btn accent wide" onclick={install} disabled={running}>
        {#if running}<ProgressRing size={16} />Kuruluyor…{:else}{error ? "Yeniden dene" : "Kur"}{/if}
      </button>
    {/if}
  </div>
</div>

{#snippet toolState(done: boolean, busy: boolean)}
  {#if done}
    <span class="status ok"><Icon name="completed" size={16} />Hazır</span>
  {:else if busy}
    <span class="status"><ProgressRing size={16} />{progressText()}</span>
  {:else}
    <span class="status text-3">{running ? "Sırada" : "Gerekli"}</span>
  {/if}
{/snippet}

<style>
  .page {
    max-width: 600px;
    margin: 0 auto;
    padding: 48px 32px 32px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    animation: enter 0.4s var(--ease-out);
  }
  .hero {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    margin-bottom: 12px;
  }
  .hero img {
    margin-bottom: 8px;
  }
  h1,
  p {
    margin: 0;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-variant-numeric: tabular-nums;
    color: var(--text-2);
  }
  .status.ok {
    color: var(--text-1);
  }
  .status.ok :global(.glyph) {
    color: var(--success);
  }
  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 8px;
  }
  .wide {
    min-width: 120px;
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
</style>
