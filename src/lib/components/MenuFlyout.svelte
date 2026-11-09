<!--
  Fluent MenuFlyout used only when a native menu can't be shown
  (browser preview, or if the OS menu fails).
-->
<script lang="ts">
  import { tick } from "svelte";
  import { fallbackMenu, type MenuEntry } from "$lib/native.svelte";

  let el: HTMLDivElement | undefined = $state();
  let pos = $state({ x: 0, y: 0 });

  const menu = $derived(fallbackMenu.current);

  $effect(() => {
    if (!menu) return;
    pos = { x: menu.x, y: menu.y };
    tick().then(() => {
      if (!el) return;
      const r = el.getBoundingClientRect();
      pos = {
        x: Math.max(4, Math.min(menu.x, window.innerWidth - r.width - 4)),
        y: Math.max(4, Math.min(menu.y, window.innerHeight - r.height - 4)),
      };
      el.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
    });
  });

  const close = () => (fallbackMenu.current = null);

  function run(entry: MenuEntry) {
    close();
    if (entry !== "separator" && "action" in entry) entry.action();
  }

  function onkeydown(e: KeyboardEvent) {
    if (!el) return;
    const buttons = [...el.querySelectorAll<HTMLButtonElement>("button:not(:disabled)")];
    const i = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (e.key === "Escape") close();
    else if (e.key === "ArrowDown") buttons[(i + 1) % buttons.length]?.focus();
    else if (e.key === "ArrowUp") buttons[(i - 1 + buttons.length) % buttons.length]?.focus();
    else return;
    e.preventDefault();
  }
</script>

<svelte:window
  onpointerdown={(e) => menu && !el?.contains(e.target as Node) && close()}
  onblur={close}
  onresize={close}
/>

{#if menu}
  <div
    bind:this={el}
    class="flyout"
    role="menu"
    tabindex="-1"
    style:left="{pos.x}px"
    style:top="{pos.y}px"
    {onkeydown}
  >
    {#each menu.entries as entry, i (i)}
      {#if entry === "separator"}
        <div class="separator" role="separator"></div>
      {:else}
        <button role="menuitem" disabled={entry.enabled === false} onclick={() => run(entry)}>
          {entry.text}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .flyout {
    position: fixed;
    z-index: 100;
    min-width: 200px;
    padding: 4px 0;
    display: flex;
    flex-direction: column;
    background: var(--flyout);
    border: 1px solid var(--flyout-stroke);
    border-radius: var(--radius-overlay);
    box-shadow: var(--shadow-flyout);
    animation: open 0.167s var(--ease-out);
  }
  button {
    text-align: left;
    min-height: 32px;
    margin: 0 4px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    cursor: default;
  }
  button:hover,
  button:focus-visible {
    background: var(--fill-subtle-hover);
    outline: none;
  }
  button:active {
    background: var(--fill-subtle-pressed);
  }
  button:disabled {
    color: var(--text-disabled);
    background: transparent;
  }
  .separator {
    height: 1px;
    margin: 4px 0;
    background: var(--divider);
  }
  @keyframes open {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }
</style>
