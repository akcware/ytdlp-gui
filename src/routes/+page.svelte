<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getVersion } from "@tauri-apps/api/app";
  import { ask, message } from "@tauri-apps/plugin-dialog";
  import { api, errorMessage, formatBytes, hostOf, looksLikeUrl, type VideoInfo } from "$lib/api";
  import {
    belowElement,
    notify,
    readClipboard,
    setTaskbarProgress,
    showMenu,
    type MenuEntry,
  } from "$lib/native.svelte";
  import { downloads } from "$lib/state/downloads.svelte";
  import { settings } from "$lib/state/settings.svelte";
  import { tools } from "$lib/state/tools.svelte";
  import DownloadItem from "$lib/components/DownloadItem.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import InfoBar from "$lib/components/InfoBar.svelte";
  import Onboarding from "$lib/components/Onboarding.svelte";
  import ProgressRing from "$lib/components/ProgressRing.svelte";
  import VideoPanel from "$lib/components/VideoPanel.svelte";

  type Notice = { id: number; severity: "info" | "warning" | "error" | "success"; title: string; message?: string };

  let url = $state("");
  let info = $state<VideoInfo | null>(null);
  let infoUrl = $state("");
  let fetching = $state(false);
  let fetchToken = 0;

  let notices = $state<Notice[]>([]);
  let noticeId = 0;
  let ffmpegDismissed = $state(false);
  let suggestion = $state<string | null>(null);
  let lastClipboard = "";

  let onboardingActive = $state(false);
  let scrolled = $state(false);
  let dragging = $state(false);
  let dragDepth = 0;

  let urlInput: HTMLInputElement | undefined = $state();
  let moreButton: HTMLButtonElement | undefined = $state();
  let panel: VideoPanel | undefined = $state();

  const showOnboarding = $derived(tools.status !== null && (!tools.ready || onboardingActive));
  const empty = $derived(!info && !fetching && downloads.jobs.length === 0);

  // ---- startup ------------------------------------------------------------

  onMount(() => {
    settings.init();
    tools.init().catch((e) => notice("error", "Bileşenler denetlenemedi", errorMessage(e)));
    downloads.init();

    downloads.onFinished = (job) => {
      if (document.hasFocus()) return;
      if (job.state === "done") notify("İndirme tamamlandı", job.title);
      else if (job.state === "error") notify("İndirme başarısız", job.title);
    };

    // Ask before quitting while something is still downloading.
    const unlistenClose = getCurrentWindow().onCloseRequested(async (event) => {
      const n = downloads.active.length;
      if (!n) return;
      event.preventDefault();
      const quit = await ask(
        `${n} indirme devam ediyor. Çıkarsanız ${n > 1 ? "bunlar" : "bu"} iptal edilecek.`,
        { title: "yt-dlp GUI", kind: "warning", okLabel: "Çık", cancelLabel: "İptal" },
      );
      if (quit) await getCurrentWindow().destroy();
    });

    return () => {
      unlistenClose.then((fn) => fn());
    };
  });

  // Overall progress on the taskbar button.
  $effect(() => {
    const active = downloads.active;
    if (!active.length) return setTaskbarProgress("none");
    const known = active.filter((j) => j.state === "downloading" && j.percent != null);
    if (!known.length) return setTaskbarProgress("indeterminate");
    const overall = known.map((j) =>
      j.request.playlist && j.playlistIndex && j.playlistCount
        ? ((j.playlistIndex - 1 + (j.percent ?? 0) / 100) / j.playlistCount) * 100
        : (j.percent ?? 0),
    );
    setTaskbarProgress(overall.reduce((a, b) => a + b, 0) / overall.length);
  });

  // ---- notices ----------------------------------------------------------------

  function notice(severity: Notice["severity"], title: string, message?: string) {
    const id = ++noticeId;
    notices = [...notices.filter((n) => n.title !== title), { id, severity, title, message }];
    if (severity === "success" || severity === "info") {
      setTimeout(() => dismiss(id), 6000);
    }
  }
  const dismiss = (id: number) => (notices = notices.filter((n) => n.id !== id));

  // ---- fetching ---------------------------------------------------------------

  async function fetchInfo(target = url) {
    target = target.trim();
    if (!target || !tools.ready) return;
    if (!looksLikeUrl(target)) {
      notice("warning", "Bu bir bağlantıya benzemiyor", "http:// veya https:// ile başlayan bir adres yapıştırın.");
      return;
    }
    url = target;
    suggestion = null;
    const token = ++fetchToken;
    fetching = true;
    notices = notices.filter((n) => n.severity !== "error");
    try {
      const result = await api.fetchInfo(target);
      if (token !== fetchToken) return;
      info = result;
      infoUrl = target;
    } catch (e) {
      if (token !== fetchToken) return;
      info = null;
      notice("error", "Video bilgisi alınamadı", errorMessage(e));
    } finally {
      if (token === fetchToken) fetching = false;
    }
  }

  function clearVideo() {
    fetchToken++;
    fetching = false;
    info = null;
    url = "";
    urlInput?.focus();
  }

  async function pasteAndFetch() {
    const text = (await readClipboard()).trim();
    if (!text) return urlInput?.focus();
    lastClipboard = text;
    url = text;
    fetchInfo(text);
  }

  function onpaste(e: ClipboardEvent) {
    const text = e.clipboardData?.getData("text")?.trim() ?? "";
    if (!looksLikeUrl(text) || (url && urlInput && urlInput.selectionEnd! - urlInput.selectionStart! < url.length))
      return;
    // Replacing the whole field with a link: fetch right away.
    e.preventDefault();
    lastClipboard = text;
    url = text;
    fetchInfo(text);
  }

  // Offer a copied link when the user comes back to the window.
  async function onfocus() {
    if (!tools.ready || showOnboarding) return;
    const text = (await readClipboard()).trim();
    if (!looksLikeUrl(text) || text === lastClipboard || text === url || text === infoUrl) return;
    lastClipboard = text;
    suggestion = text;
  }

  // ---- drag & drop links ------------------------------------------------------

  const hasLink = (e: DragEvent) =>
    !!e.dataTransfer && [...e.dataTransfer.types].some((t) => t === "text/uri-list" || t === "text/plain");

  function ondragenter(e: DragEvent) {
    if (!hasLink(e) || !tools.ready) return;
    e.preventDefault();
    dragDepth++;
    dragging = true;
  }
  function ondragover(e: DragEvent) {
    if (!dragging) return;
    e.preventDefault();
    e.dataTransfer!.dropEffect = "link";
  }
  function ondragleave() {
    if (--dragDepth <= 0) {
      dragDepth = 0;
      dragging = false;
    }
  }
  function ondrop(e: DragEvent) {
    e.preventDefault();
    dragDepth = 0;
    dragging = false;
    const text =
      e.dataTransfer?.getData("text/uri-list").split("\n").find((l) => l && !l.startsWith("#")) ||
      e.dataTransfer?.getData("text/plain") ||
      "";
    if (text) fetchInfo(text.trim());
  }

  // ---- keyboard ---------------------------------------------------------------

  function onkeydown(e: KeyboardEvent) {
    if (showOnboarding) return;
    const ctrl = e.ctrlKey || e.metaKey;
    const inField = (e.target as HTMLElement).closest("input, textarea");
    if (ctrl && e.key.toLowerCase() === "l") {
      e.preventDefault();
      urlInput?.focus();
      urlInput?.select();
    } else if (ctrl && e.key.toLowerCase() === "v" && !inField) {
      e.preventDefault();
      pasteAndFetch();
    } else if (ctrl && e.key === "Enter" && info) {
      e.preventDefault();
      panel?.download();
    } else if (e.key === "Escape" && info && !e.defaultPrevented) {
      clearVideo();
    }
  }

  // ---- "…" menu -----------------------------------------------------------------

  async function about() {
    const version = await getVersion().catch(() => "?");
    await message(
      `yt-dlp GUI ${version}\n\n` +
        `yt-dlp ${tools.status?.ytdlpVersion ?? "yüklü değil"}\n` +
        `ffmpeg ${tools.status?.ffmpeg ? "yüklü" : "yüklü değil"}\n\n` +
        `yt-dlp (Unlicense) ve ffmpeg (GPL) kendi lisanslarıyla, kendi projelerinden indirilir.`,
      { title: "Hakkında", kind: "info" },
    );
  }

  async function updateYtdlp() {
    try {
      notice("success", await tools.update());
    } catch (e) {
      notice("error", "yt-dlp güncellenemedi", String(e));
    }
  }

  async function installFfmpeg() {
    try {
      await tools.install("ffmpeg");
      notice("success", "ffmpeg hazır", "Artık en yüksek kalitede video ve MP3 indirebilirsiniz.");
    } catch (e) {
      notice("error", "ffmpeg indirilemedi", String(e));
    }
  }

  function openFolder(path: string | undefined) {
    if (path) api.openPath(path).catch((e) => notice("error", "Klasör açılamadı", errorMessage(e)));
  }

  function moreMenu() {
    if (!moreButton) return;
    const s = tools.status;
    const entries: MenuEntry[] = [
      { text: `yt-dlp ${s?.ytdlpVersion ?? "yüklü değil"}`, enabled: false },
      { text: s?.ffmpeg ? "ffmpeg yüklü" : "ffmpeg yüklü değil", enabled: false },
      "separator",
      { text: "yt-dlp'yi güncelle", action: updateYtdlp, enabled: !tools.busy },
    ];
    if (s && !s.ffmpeg) entries.push({ text: "ffmpeg'i indir", action: installFfmpeg, enabled: !tools.busy });
    entries.push(
      "separator",
      { text: "İndirme klasörünü aç", action: () => openFolder(settings.outDir) },
      { text: "Bileşen klasörünü aç", action: () => openFolder(s?.binDir) },
      "separator",
      { text: "Hakkında", action: about },
    );
    const at = belowElement(moreButton);
    showMenu(entries, { x: at.x + moreButton.offsetWidth - 220, y: at.y });
  }

  function focusOnLoad(node: HTMLInputElement) {
    tick().then(() => node.focus());
  }
</script>

<svelte:window {onkeydown} {onfocus} {ondragenter} {ondragover} {ondragleave} {ondrop} />

{#if tools.status === null}
  <!-- checking components: keep the window calm, no spinner flash -->
  <div class="splash"></div>
{:else if showOnboarding}
  <main class="scroll fill">
    <Onboarding bind:active={onboardingActive} />
  </main>
{:else}
  <div class="shell">
    <header class="commandbar" class:scrolled>
      <div class="textbox url">
        <Icon name="link" />
        <input
          bind:this={urlInput}
          bind:value={url}
          use:focusOnLoad
          type="text"
          placeholder="Video, playlist veya kanal bağlantısı yapıştırın"
          aria-label="Bağlantı"
          spellcheck="false"
          autocomplete="off"
          {onpaste}
          onkeydown={(e) => e.key === "Enter" && !e.ctrlKey && !e.metaKey && fetchInfo()}
        />
        {#if url}
          <button class="btn subtle icon" title="Temizle" aria-label="Temizle" onclick={clearVideo}>
            <Icon name="cancel" size={12} />
          </button>
        {:else}
          <button class="btn subtle icon" title="Yapıştır (Ctrl+V)" aria-label="Yapıştır" onclick={pasteAndFetch}>
            <Icon name="paste" size={14} />
          </button>
        {/if}
      </div>
      <button class="btn accent fetch" onclick={() => fetchInfo()} disabled={fetching || !url.trim()}>
        {#if fetching}<ProgressRing size={16} />{:else}Getir{/if}
      </button>
      <button
        bind:this={moreButton}
        class="btn subtle icon"
        title="Diğer seçenekler"
        aria-label="Diğer seçenekler"
        onclick={moreMenu}
      >
        <Icon name="more" />
      </button>

      <!-- Floats over the content (like a WinUI TeachingTip) so it never shifts
           what's under the cursor when the window regains focus. -->
      {#if suggestion}
        <div class="suggestion" role="status">
          <Icon name="paste" size={14} />
          <span class="ellipsis">Panoda bir bağlantı var: <span class="text-2">{hostOf(suggestion) || suggestion}</span></span>
          <button class="btn small accent" onclick={() => fetchInfo(suggestion!)}>Getir</button>
          <button class="btn small subtle icon" aria-label="Yoksay" title="Yoksay" onclick={() => (suggestion = null)}>
            <Icon name="cancel" size={10} />
          </button>
        </div>
      {/if}
    </header>

    <main class="scroll" onscroll={(e) => (scrolled = e.currentTarget.scrollTop > 0)}>
      <div class="content">

        {#if tools.status && !tools.status.ffmpeg && !ffmpegDismissed}
          <InfoBar
            severity="warning"
            title="ffmpeg yüklü değil"
            message="720p üstü video ve MP3 dönüştürme için gerekli."
            onclose={tools.busy === "ffmpeg" ? undefined : () => (ffmpegDismissed = true)}
          >
            {#snippet action()}
              {#if tools.busy === "ffmpeg"}
                <span class="busy caption">
                  <ProgressRing size={16} />
                  {tools.phase === "extract" ? "Arşiv açılıyor…" : formatBytes(tools.bytes)}
                </span>
              {:else}
                <button class="btn" onclick={installFfmpeg} disabled={!!tools.busy}>İndir</button>
              {/if}
            {/snippet}
          </InfoBar>
        {/if}

        {#each notices as n (n.id)}
          <InfoBar severity={n.severity} title={n.title} message={n.message} onclose={() => dismiss(n.id)} />
        {/each}

        {#if fetching && !info}
          <section class="card skeleton" aria-busy="true" aria-label="Video bilgisi alınıyor">
            <div class="shimmer thumb"></div>
            <div class="lines">
              <div class="shimmer line" style:width="70%"></div>
              <div class="shimmer line" style:width="45%"></div>
              <div class="shimmer line small" style:width="25%"></div>
            </div>
          </section>
        {/if}

        {#if info}
          {#key infoUrl}
            <div class="panel">
              <VideoPanel bind:this={panel} {info} url={infoUrl} onclose={clearVideo} />
            </div>
          {/key}
        {/if}

        {#if empty}
          <div class="empty">
            <div class="empty-icon"><Icon name="download" size={32} /></div>
            <div class="subtitle">Bir bağlantı yapıştırarak başlayın</div>
            <p class="text-2">
              YouTube ve yüzlerce siteden video ya da ses indirin. Bağlantıyı
              <kbd>Ctrl</kbd>+<kbd>V</kbd> ile yapıştırabilir ya da pencereye sürükleyebilirsiniz.
            </p>
            <button class="btn" onclick={pasteAndFetch}><Icon name="paste" />Panodan yapıştır</button>
          </div>
        {/if}

        {#if downloads.jobs.length}
          <div class="list-header">
            <span class="group-header">İndirmeler</span>
            {#if downloads.finished.length}
              <button class="link" onclick={() => downloads.clearFinished()}>Bitenleri temizle</button>
            {/if}
          </div>
          <div class="list" role="list">
            {#each downloads.jobs as job (job.key)}
              <DownloadItem {job} onerror={(m) => notice("error", "Açılamadı", m)} />
            {/each}
          </div>
        {/if}
      </div>
    </main>
  </div>
{/if}

{#if dragging}
  <div class="dropzone" aria-hidden="true">
    <div class="drop-card">
      <Icon name="link" size={28} />
      <span class="subtitle">Bağlantıyı bırakın</span>
    </div>
  </div>
{/if}

<style>
  .splash,
  .fill,
  .shell {
    height: 100vh;
  }
  .shell {
    display: flex;
    flex-direction: column;
  }

  .commandbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    border-bottom: 1px solid transparent;
    transition: border-color 0.15s;
  }
  .commandbar.scrolled {
    border-bottom-color: var(--divider);
  }
  .url {
    flex: 1;
  }
  .fetch {
    min-width: 80px;
  }

  main.scroll {
    flex: 1;
    min-height: 0;
  }
  .content {
    max-width: 880px;
    margin: 0 auto;
    padding: 8px 24px 32px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .panel {
    display: flex;
    flex-direction: column;
    margin-bottom: 8px;
    animation: rise 0.3s var(--ease-out);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }

  .commandbar {
    position: relative;
    z-index: 10;
  }
  .suggestion {
    position: absolute;
    top: calc(100% - 6px);
    left: 24px;
    max-width: min(520px, calc(100% - 48px));
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 6px 6px 12px;
    box-sizing: border-box;
    border-radius: var(--radius-overlay);
    background: var(--flyout);
    border: 1px solid var(--flyout-stroke);
    box-shadow: var(--shadow-flyout);
    animation: enter 0.2s var(--ease-out);
  }
  .suggestion > .ellipsis {
    flex: 1;
  }
  .suggestion :global(.glyph) {
    color: var(--text-2);
  }
  .busy {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
  }

  .skeleton {
    display: flex;
    gap: 16px;
    padding: 12px;
  }
  .skeleton .thumb {
    width: 208px;
    aspect-ratio: 16 / 9;
    border-radius: var(--radius-control);
  }
  .lines {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-top: 6px;
  }
  .line {
    height: 18px;
    border-radius: var(--radius-control);
  }
  .line.small {
    height: 12px;
  }
  .shimmer {
    background: linear-gradient(
      90deg,
      var(--fill-alt) 0%,
      var(--fill-subtle-hover) 40%,
      var(--fill-alt) 80%
    );
    background-size: 300% 100%;
    animation: shimmer 1.4s linear infinite;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 8px;
    padding: 72px 24px 24px;
    animation: enter 0.4s var(--ease-out);
  }
  .empty-icon {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    margin-bottom: 8px;
    color: var(--accent-text);
    background: color-mix(in srgb, var(--accent) 16%, transparent);
  }
  .empty p {
    margin: 0 0 12px;
    max-width: 420px;
  }
  kbd {
    font: 600 12px/1 var(--font-small);
    padding: 2px 5px;
    border-radius: 3px;
    border: 1px solid var(--stroke-control-2);
    background: var(--fill-control);
  }

  .list-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .list-header .link {
    margin-right: -8px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .dropzone {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: grid;
    place-items: center;
    background: var(--smoke);
    pointer-events: none;
    animation: fade 0.15s var(--ease-out);
  }
  .drop-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 32px 48px;
    border-radius: var(--radius-overlay);
    background: var(--flyout);
    border: 2px dashed var(--accent);
    box-shadow: var(--shadow-flyout);
    color: var(--accent-text);
  }
  .drop-card .subtitle {
    color: var(--text-1);
  }

  @media (max-width: 640px) {
    .commandbar,
    .content {
      padding-left: 16px;
      padding-right: 16px;
    }
    .skeleton .thumb {
      width: 136px;
    }
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes shimmer {
    from {
      background-position: 100% 0;
    }
    to {
      background-position: -50% 0;
    }
  }
</style>
