<!-- WinUI ProgressBar: 1px track, 3px rounded indicator, two-bar indeterminate motion.
     On macOS: a 5px rounded track, like NSProgressIndicator. -->
<script lang="ts">
  let {
    value = null,
    status = "normal",
  }: {
    /** 0–100, or null for indeterminate */
    value?: number | null;
    status?: "normal" | "paused" | "error";
  } = $props();
</script>

<div
  class="progress {status}"
  class:indeterminate={value == null}
  role="progressbar"
  aria-valuemin="0"
  aria-valuemax="100"
  aria-valuenow={value ?? undefined}
>
  {#if value == null}
    <div class="bar a"></div>
    <div class="bar b"></div>
  {:else}
    <div class="bar" style:width="{Math.max(0, Math.min(100, value))}%"></div>
  {/if}
</div>

<style>
  .progress {
    position: relative;
    height: 3px;
    overflow: hidden;
    border-radius: 2px;
  }
  .progress::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 1px;
    height: 1px;
    background: var(--stroke-strong);
  }
  .bar {
    position: absolute;
    left: 0;
    top: 0;
    height: 3px;
    border-radius: 2px;
    background: var(--accent);
    transition: width 0.25s var(--ease-out);
  }
  .paused .bar {
    background: var(--caution);
  }
  .error .bar {
    background: var(--critical);
  }

  .indeterminate .bar {
    transition: none;
  }
  .indeterminate .a {
    width: 40%;
    animation: a 2s infinite;
  }
  .indeterminate .b {
    width: 60%;
    animation: b 2s infinite;
  }
  @keyframes a {
    0% {
      transform: translateX(-100%);
      animation-timing-function: cubic-bezier(0.4, 0, 0.6, 1);
    }
    50%,
    100% {
      transform: translateX(260%);
    }
  }
  @keyframes b {
    0%,
    45% {
      transform: translateX(-170%);
      animation-timing-function: cubic-bezier(0.4, 0, 0.6, 1);
    }
    100% {
      transform: translateX(170%);
    }
  }

  :global(.macos) .progress {
    height: 5px;
    border-radius: 3px;
    background: var(--fill-track);
  }
  :global(.macos) .progress::before {
    content: none;
  }
  :global(.macos) .bar {
    height: 5px;
    border-radius: 3px;
  }
</style>
