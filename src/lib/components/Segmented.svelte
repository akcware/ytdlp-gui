<!-- Segmented control (Windows Community Toolkit style) with roving focus. -->
<script lang="ts" generics="T">
  import Icon, { type IconName } from "./Icon.svelte";

  type Item = { value: T; label: string; icon?: IconName };
  let { items, value = $bindable(), label }: { items: Item[]; value: T; label: string } = $props();

  let buttons: HTMLButtonElement[] = $state([]);

  function onkeydown(e: KeyboardEvent, i: number) {
    const delta = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 0;
    if (!delta) return;
    e.preventDefault();
    const next = (i + delta + items.length) % items.length;
    value = items[next].value;
    buttons[next]?.focus();
  }
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
  {#each items as item, i (i)}
    {@const active = Object.is(item.value, value)}
    <button
      bind:this={buttons[i]}
      role="radio"
      aria-checked={active}
      tabindex={active ? 0 : -1}
      class:active
      onclick={() => (value = item.value)}
      onkeydown={(e) => onkeydown(e, i)}
    >
      {#if item.icon}<Icon name={item.icon} size={14} />{/if}
      {item.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border-radius: 6px;
    border: 1px solid var(--stroke-control);
    background: var(--fill-alt);
  }
  button {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 28px;
    padding: 0 14px;
    border: 1px solid transparent;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-2);
    cursor: default;
  }
  button:hover:not(.active) {
    background: var(--fill-subtle-hover);
    color: var(--text-1);
  }
  button:active:not(.active) {
    background: var(--fill-subtle-pressed);
  }
  button.active {
    color: var(--text-1);
    background: var(--fill-control);
    border-color: var(--stroke-control);
    border-bottom-color: var(--stroke-control-2);
    background-clip: padding-box;
  }
  button.active::after {
    content: "";
    position: absolute;
    left: 50%;
    bottom: 1px;
    width: 12px;
    height: 3px;
    margin-left: -6px;
    border-radius: 2px;
    background: var(--accent);
  }

  /* macOS segmented control: a raised white segment on a recessed track */
  :global(.macos) .segmented {
    gap: 0;
    padding: 1px;
    border: 0;
    border-radius: 7px;
    background: var(--fill-alt);
    box-shadow: inset 0 0 0 0.5px var(--stroke-card);
  }
  :global(.macos) button {
    gap: 5px;
    min-height: 22px;
    padding: 0 12px;
    border: 0;
    border-radius: 6px;
    color: var(--text-1);
  }
  :global(.macos) button:hover:not(.active),
  :global(.macos) button:active:not(.active) {
    background: transparent;
  }
  :global(.macos) button.active {
    background: var(--fill-control);
    box-shadow: var(--shadow-button);
  }
  :global(.macos) button.active::after {
    content: none;
  }
</style>
