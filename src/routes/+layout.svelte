<script lang="ts">
  import "$lib/styles/fluent.css";
  import { onMount, tick, type Snippet } from "svelte";
  import { api } from "$lib/api";
  import { showWindow } from "$lib/native.svelte";
  import { i18n } from "$lib/i18n.svelte";
  import MenuFlyout from "$lib/components/MenuFlyout.svelte";

  let { children }: { children: Snippet } = $props();

  /** Mica + accent color, applied before the window becomes visible. */
  async function applyAppearance() {
    try {
      const a = await Promise.race([
        api.appearance(),
        new Promise<never>((_, reject) => setTimeout(() => reject(new Error("timeout")), 1500)),
      ]);
      const root = document.documentElement;
      root.classList.toggle("mica", a.backdrop === "mica");
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

  onMount(async () => {
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
    if (
      k === "f5" ||
      k === "f3" ||
      k === "f7" ||
      (ctrl && ["r", "p", "f", "g", "u", "s", "j", "h", "o", "+", "-", "=", "0"].includes(k)) ||
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

<svelte:window onkeydowncapture={onkeydown} {oncontextmenu} />

{@render children()}
<MenuFlyout />
