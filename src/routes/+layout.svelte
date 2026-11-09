<script lang="ts">
  import "$lib/styles/fluent.css";
  import "$lib/styles/macos.css";
  import { onMount, tick, type Snippet } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api } from "$lib/api";
  import { isMac } from "$lib/platform";
  import { isMock, showWindow } from "$lib/native.svelte";
  import { i18n } from "$lib/i18n.svelte";
  import MenuFlyout from "$lib/components/MenuFlyout.svelte";

  let { children }: { children: Snippet } = $props();

  /** Mica/vibrancy + accent color, applied before the window becomes visible. */
  async function applyAppearance() {
    try {
      const a = await Promise.race([
        api.appearance(),
        new Promise<never>((_, reject) => setTimeout(() => reject(new Error("timeout")), 1500)),
      ]);
      const root = document.documentElement;
      root.classList.toggle("mica", a.backdrop === "mica");
      root.classList.toggle("vibrancy", a.backdrop === "vibrancy");
      root.style.setProperty("--accent-light3", a.accent.light3);
      root.style.setProperty("--accent-light2", a.accent.light2);
      root.style.setProperty("--accent-dark1", a.accent.dark1);
      root.style.setProperty("--accent-dark2", a.accent.dark2);
    } catch {
      // keep the solid background and default accent
    }
  }

  // Screen readers and spell-checking follow the app language.
  $effect(() => {
    document.documentElement.lang = i18n.locale;
  });

  // macOS: in full screen the traffic lights are gone, so the toolbar can use
  // their space.
  async function syncFullscreen() {
    if (!isMac || isMock()) return;
    const full = await getCurrentWindow().isFullscreen().catch(() => false);
    document.documentElement.classList.toggle("fullscreen", full);
  }

  onMount(async () => {
    syncFullscreen();
    await applyAppearance();
    await tick();
    // Show only after the first frame is painted: no white flash on launch.
    requestAnimationFrame(() => requestAnimationFrame(() => showWindow()));
  });

  // Browser shortcuts that make no sense in a desktop app.
  function onkeydown(e: KeyboardEvent) {
    if (import.meta.env.DEV) return; // keep reload/devtools while developing
    const k = e.key.toLowerCase();
    const ctrl = e.ctrlKey || e.metaKey;
    // On macOS ⌘H hides the app and ⌃⌘F enters full screen: leave those
    // to the menu bar.
    const browserKeys = isMac
      ? ["r", "p", "g", "u", "s", "j", "o", "+", "-", "=", "0"]
      : ["r", "p", "f", "g", "u", "s", "j", "h", "o", "+", "-", "=", "0"];
    if (
      k === "f5" ||
      k === "f3" ||
      k === "f7" ||
      (ctrl && browserKeys.includes(k)) ||
      (isMac && e.metaKey && !e.ctrlKey && k === "f") ||
      (e.altKey && (k === "arrowleft" || k === "arrowright"))
    ) {
      e.preventDefault();
    }
  }

  // Only text fields get a (system) context menu; everything else either
  // shows its own native menu or nothing — never the browser's.
  function oncontextmenu(e: MouseEvent) {
    const t = e.target as HTMLElement;
    if (t.closest("input, textarea, .selectable")) return;
    e.preventDefault();
  }
</script>

<svelte:window onkeydowncapture={onkeydown} {oncontextmenu} onresize={syncFullscreen} />

{#if isMac}
  <!-- The title bar is hidden on macOS: keep the top of the window draggable. -->
  <div class="titlebar-drag" data-tauri-drag-region></div>
{/if}
{@render children()}
<MenuFlyout />
