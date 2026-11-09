<!-- WinUI InfoBar: severity icon, title + message, optional action and close. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { GLYPHS } from "./Icon.svelte";

  let {
    severity = "info",
    title,
    message = "",
    onclose,
    action,
  }: {
    severity?: "info" | "warning" | "error" | "success";
    title: string;
    message?: string;
    onclose?: () => void;
    action?: Snippet;
  } = $props();

  const glyph = $derived(
    severity === "success" ? GLYPHS.check : severity === "error" ? GLYPHS.cancel : null,
  );
</script>

<div class="infobar {severity}" role={severity === "error" ? "alert" : "status"}>
  <span class="severity" aria-hidden="true">
    {#if glyph}<span class="glyph">{glyph}</span>{:else}{severity === "info" ? "i" : "!"}{/if}
  </span>
  <div class="content">
    <span class="body-strong">{title}</span>
    {#if message}<span class="message selectable">{message}</span>{/if}
  </div>
  {#if action}<div class="action">{@render action()}</div>{/if}
  {#if onclose}
    <button class="btn subtle icon close" aria-label="Kapat" title="Kapat" onclick={onclose}>
      <span class="glyph" style:font-size="12px">{GLYPHS.cancel}</span>
    </button>
  {/if}
</div>

<style>
  .infobar {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 6px 4px 6px 16px;
    min-height: 48px;
    box-sizing: border-box;
    border-radius: var(--radius-control);
    border: 1px solid var(--stroke-card);
    background: var(--attention-bg);
    background-clip: padding-box;
    animation: enter 0.25s var(--ease-out);
  }
  .warning {
    background-color: var(--caution-bg);
  }
  .error {
    background-color: var(--critical-bg);
  }
  .success {
    background-color: var(--success-bg);
  }
  .severity {
    width: 16px;
    height: 16px;
    margin-top: 10px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    font: 700 11px/1 var(--font);
    color: var(--text-inverse);
    background: var(--accent);
  }
  .severity .glyph {
    font-size: 8px;
    font-weight: bold;
  }
  .warning .severity {
    background: var(--caution);
  }
  .error .severity {
    background: var(--critical);
  }
  .success .severity {
    background: var(--success);
  }
  .content {
    flex: 1;
    min-width: 0;
    padding: 8px 0;
    display: flex;
    flex-wrap: wrap;
    column-gap: 12px;
    row-gap: 2px;
  }
  .message {
    color: var(--text-1);
    white-space: pre-wrap;
    word-break: break-word;
  }
  .action {
    align-self: center;
    flex-shrink: 0;
  }
  .close {
    flex-shrink: 0;
    margin-top: 2px;
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }
</style>
