<!--
  WinUI ComboBox: button-like trigger with chevron and a flyout list where the
  selected item carries an accent pill. Keyboard: ↑/↓ change the selection
  while closed (like WinUI), Alt+↓ / Enter / Space open, Esc closes.
-->
<script lang="ts" generics="T">
  import { tick } from "svelte";
  import { GLYPHS } from "./Icon.svelte";

  type Item = { value: T; label: string; detail?: string };

  let {
    items,
    value = $bindable(),
    disabled = false,
    label,
    minWidth = 220,
  }: {
    items: Item[];
    value: T;
    disabled?: boolean;
    label: string;
    minWidth?: number;
  } = $props();

  const uid = Math.random().toString(36).slice(2, 8);
  let trigger: HTMLButtonElement;
  let list: HTMLUListElement | undefined = $state();
  let open = $state(false);
  let highlighted = $state(0);
  let pos = $state({ left: 0, top: 0, width: 0, above: false });

  const selectedIndex = $derived(items.findIndex((i) => Object.is(i.value, value)));
  const selected = $derived(items[selectedIndex]);

  async function show() {
    if (disabled || open) return;
    const r = trigger.getBoundingClientRect();
    const height = Math.min(items.length * 36 + 8, 360);
    const below = window.innerHeight - r.bottom;
    const above = below < height + 12 && r.top > below;
    pos = { left: r.left, top: above ? r.top - 4 : r.bottom + 4, width: r.width, above };
    highlighted = Math.max(0, selectedIndex);
    open = true;
    await tick();
    list?.querySelector(".highlighted")?.scrollIntoView({ block: "nearest" });
  }

  function hide(refocus = true) {
    if (!open) return;
    open = false;
    if (refocus) trigger.focus();
  }

  function choose(i: number) {
    value = items[i].value;
    hide();
  }

  function move(delta: number) {
    const last = items.length - 1;
    if (open) {
      highlighted = Math.max(0, Math.min(last, highlighted + delta));
      tick().then(() => list?.querySelector(".highlighted")?.scrollIntoView({ block: "nearest" }));
    } else {
      const next = Math.max(0, Math.min(last, Math.max(0, selectedIndex) + delta));
      value = items[next].value;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    // Leave shortcuts like Ctrl+Enter to the page.
    if (e.ctrlKey || e.metaKey) return;
    switch (e.key) {
      case "ArrowDown":
        if (e.altKey && !open) show();
        else move(1);
        break;
      case "ArrowUp":
        move(-1);
        break;
      case "Home":
        open ? (highlighted = 0) : (value = items[0].value);
        break;
      case "End":
        open ? (highlighted = items.length - 1) : (value = items[items.length - 1].value);
        break;
      case "Enter":
      case " ":
        open ? choose(highlighted) : show();
        break;
      case "Escape":
        if (!open) return;
        hide();
        break;
      case "Tab":
        hide(false);
        return;
      default:
        return;
    }
    e.preventDefault();
    e.stopPropagation();
  }

  function onWindowPointerDown(e: PointerEvent) {
    if (!open) return;
    const target = e.target as Node;
    if (!trigger.contains(target) && !list?.contains(target)) hide(false);
  }
</script>

<svelte:window
  onpointerdown={onWindowPointerDown}
  onblur={() => hide(false)}
  onresize={() => hide(false)}
/>

<button
  bind:this={trigger}
  class="btn combo"
  style:min-width="{minWidth}px"
  role="combobox"
  aria-label={label}
  aria-expanded={open}
  aria-controls="lb-{uid}"
  aria-activedescendant={open ? `lb-${uid}-${highlighted}` : undefined}
  {disabled}
  onclick={() => (open ? hide() : show())}
  {onkeydown}
>
  <span class="ellipsis">{selected?.label ?? ""}</span>
  {#if selected?.detail}<span class="caption text-2 detail">{selected.detail}</span>{/if}
  <span class="glyph chevron">{GLYPHS.chevronDown}</span>
</button>

{#if open}
  <ul
    bind:this={list}
    id="lb-{uid}"
    class="popup scroll"
    class:above={pos.above}
    role="listbox"
    aria-label={label}
    style:left="{pos.left}px"
    style:top={pos.above ? undefined : `${pos.top}px`}
    style:bottom={pos.above ? `${window.innerHeight - pos.top}px` : undefined}
    style:min-width="{pos.width}px"
  >
    {#each items as item, i (i)}
      <!-- svelte-ignore a11y_click_events_have_key_events (keyboard handled on the combobox) -->
      <li
        id="lb-{uid}-{i}"
        role="option"
        aria-selected={i === selectedIndex}
        class:selected={i === selectedIndex}
        class:highlighted={i === highlighted}
        onpointermove={() => (highlighted = i)}
        onclick={() => choose(i)}
      >
        <span class="label">{item.label}</span>
        {#if item.detail}<span class="caption text-2">{item.detail}</span>{/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
  .combo {
    justify-content: flex-start;
    gap: 8px;
    padding: 0 10px 0 11px;
    max-width: 100%;
  }
  .combo .detail {
    margin-left: auto;
  }
  .chevron {
    font-size: 12px;
    margin-left: auto;
    padding-left: 4px;
    color: var(--text-2);
  }
  .detail + .chevron {
    margin-left: 0;
  }

  .popup {
    position: fixed;
    z-index: 50;
    margin: 0;
    padding: 2px 0;
    list-style: none;
    max-height: 360px;
    box-sizing: border-box;
    background: var(--flyout);
    border: 1px solid var(--flyout-stroke);
    border-radius: var(--radius-overlay);
    box-shadow: var(--shadow-flyout);
    animation: drop 0.167s var(--ease-out);
  }
  .popup.above {
    animation-name: rise;
  }
  li {
    position: relative;
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 32px;
    margin: 2px 4px;
    padding: 0 12px;
    border-radius: var(--radius-control);
    white-space: nowrap;
  }
  li .caption {
    margin-left: auto;
  }
  li.highlighted {
    background: var(--fill-subtle-hover);
  }
  li.selected {
    background: var(--fill-subtle-hover);
  }
  li.selected.highlighted {
    background: var(--fill-subtle-pressed);
  }
  li.selected::before {
    content: "";
    position: absolute;
    left: 0;
    top: 50%;
    width: 3px;
    height: 16px;
    margin-top: -8px;
    border-radius: 2px;
    background: var(--accent);
  }
  @keyframes drop {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
