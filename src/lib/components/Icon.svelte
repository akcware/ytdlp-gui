<script lang="ts" module>
  // Segoe Fluent Icons code points (same in Segoe MDL2 Assets on Windows 10).
  export const GLYPHS = {
    add: "",
    audio: "",
    cancel: "",
    check: "",
    chevronDown: "",
    clear: "",
    completed: "",
    copy: "",
    download: "",
    error: "",
    folder: "",
    folderOpen: "",
    globe: "",
    info: "",
    link: "",
    list: "",
    more: "",
    openFile: "",
    paste: "",
    play: "",
    refresh: "",
    settings: "",
    sync: "",
    video: "",
    warning: "",
  } as const;
  export type IconName = keyof typeof GLYPHS | "chevronUpDown";

  // Segoe Fluent Icons doesn't exist on macOS: there the same icons are drawn
  // as outline SVGs in the spirit of SF Symbols (24×24 grid, round caps).
  // `chevronUpDown` is the macOS pop-up button indicator; Windows uses chevronDown.
  const SVG: Record<IconName, string> = {
    add: `<path d="M12 5v14M5 12h14"/>`,
    audio: `<path d="M9 17.5V5.5l10.5-2v12"/><circle cx="6.5" cy="17.5" r="2.5"/><circle cx="17" cy="15.5" r="2.5"/>`,
    cancel: `<path d="M6.5 6.5l11 11M17.5 6.5l-11 11"/>`,
    check: `<path d="M5 12.5l4.5 4.5L19 7.5"/>`,
    chevronDown: `<path d="M6.5 9.5l5.5 5.5 5.5-5.5"/>`,
    chevronUpDown: `<path d="M8 9.5l4-4 4 4M8 14.5l4 4 4-4"/>`,
    clear: `<path d="M6.5 6.5l11 11M17.5 6.5l-11 11"/>`,
    completed: `<circle cx="12" cy="12" r="9"/><path d="M8 12.3l2.8 2.8L16.2 9.6"/>`,
    copy: `<rect x="8.5" y="8.5" width="11.5" height="11.5" rx="2.5"/><path d="M15.5 8.5V6.5A2.5 2.5 0 0 0 13 4H6.5A2.5 2.5 0 0 0 4 6.5V13a2.5 2.5 0 0 0 2.5 2.5h2"/>`,
    download: `<path d="M12 4v11M7 10.5l5 5 5-5M5 20h14"/>`,
    error: `<circle cx="12" cy="12" r="9"/><path d="M9 9l6 6M15 9l-6 6"/>`,
    folder: `<path d="M3.5 7.5A2 2 0 0 1 5.5 5.5h3.6l2 2h7.4a2 2 0 0 1 2 2v7.5a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/><path d="M3.5 10h17"/>`,
    folderOpen: `<path d="M4 18.5V7a2 2 0 0 1 2-2h3.3l2 2H17a2 2 0 0 1 2 2v1.5"/><path d="M4 18.5l2.2-6.6a2 2 0 0 1 1.9-1.4H20a1 1 0 0 1 .95 1.3l-2 6.1a2 2 0 0 1-1.9 1.4H4z"/>`,
    globe: `<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.6 2.6 3.8 5.6 3.8 9s-1.2 6.4-3.8 9c-2.6-2.6-3.8-5.6-3.8-9S9.4 5.6 12 3z"/>`,
    info: `<circle cx="12" cy="12" r="9"/><path d="M12 11v5.5M12 7.6v.1"/>`,
    link: `<path d="M10.6 13.4a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1.4 1.4"/><path d="M13.4 10.6a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1.4-1.4"/>`,
    list: `<path d="M9 6.5h11M9 12h11M9 17.5h11M4.5 6.5h.1M4.5 12h.1M4.5 17.5h.1"/>`,
    more: `<g fill="currentColor" stroke="none"><circle cx="5.5" cy="12" r="1.7"/><circle cx="12" cy="12" r="1.7"/><circle cx="18.5" cy="12" r="1.7"/></g>`,
    openFile: `<path d="M13.5 4.5H19.5V10.5M19.5 4.5l-8.5 8.5"/><path d="M17.5 13.5V18a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8.5a2 2 0 0 1 2-2h4.5"/>`,
    paste: `<rect x="8.5" y="3" width="7" height="3.5" rx="1"/><path d="M8.5 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-1.5"/>`,
    play: `<path d="M8 5.5v13l10-6.5z"/>`,
    refresh: `<path d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3"/><path d="M18.3 3.5v3.7h-3.7"/>`,
    settings: `<path d="M4 7h9.5M17.5 7H20M4 12h2.5M10.5 12H20M4 17h11.5M19.5 17H20"/><circle cx="15.5" cy="7" r="2"/><circle cx="8.5" cy="12" r="2"/><circle cx="17.5" cy="17" r="2"/>`,
    sync: `<path d="M4.5 12a7.5 7.5 0 0 1 12.8-5.3l2.2 2.2M19.5 4.5v4.4h-4.4M19.5 12a7.5 7.5 0 0 1-12.8 5.3l-2.2-2.2M4.5 19.5v-4.4h4.4"/>`,
    video: `<rect x="3" y="5" width="18" height="14" rx="3"/><path d="M10 9.3v5.4l4.6-2.7z"/>`,
    warning: `<path d="M10.3 4.3a2 2 0 0 1 3.4 0l7.5 13a2 2 0 0 1-1.7 3H4.5a2 2 0 0 1-1.7-3z"/><path d="M12 9.5v4.5M12 17v.1"/>`,
  };
</script>

<script lang="ts">
  import { isMac } from "$lib/platform";

  let { name, size = 16, class: cls = "" }: { name: IconName; size?: number; class?: string } =
    $props();

  // A rendered stroke of ~1.35px at any size, like SF Symbols' regular weight.
  const stroke = $derived(Math.min(2.4, Math.max(1.6, (24 * 1.35) / size)));
</script>

{#if isMac}
  <svg
    class="glyph icon {cls}"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width={stroke}
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true">{@html SVG[name]}</svg
  >
{:else}
  <span class="glyph {cls}" style:font-size="{size}px" style:width="{size}px" aria-hidden="true"
    >{GLYPHS[name === "chevronUpDown" ? "chevronDown" : name]}</span
  >
{/if}
