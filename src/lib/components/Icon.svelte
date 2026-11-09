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
  export type IconName = keyof typeof GLYPHS;
</script>

<script lang="ts">
  let { name, size = 16, class: cls = "" }: { name: IconName; size?: number; class?: string } =
    $props();
</script>

<span class="glyph {cls}" style:font-size="{size}px" style:width="{size}px" aria-hidden="true"
  >{GLYPHS[name]}</span
>
