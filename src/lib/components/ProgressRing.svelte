<!-- WinUI ProgressRing (indeterminate): a rotating arc that grows and shrinks.
     On macOS the AppKit spinner instead: spokes fading around a circle. -->
<script lang="ts">
  import { isMac } from "$lib/platform";

  let { size = 16 }: { size?: number } = $props();

  const SPOKES = 8;
</script>

{#if isMac}
  <svg class="spinner" width={size} height={size} viewBox="0 0 16 16" role="progressbar" aria-busy="true">
    {#each { length: SPOKES } as _, i}
      <line
        x1="8"
        y1="1.6"
        x2="8"
        y2="4.6"
        transform="rotate({(360 / SPOKES) * i} 8 8)"
        opacity={0.25 + (0.75 * i) / (SPOKES - 1)}
      />
    {/each}
  </svg>
{:else}
  <svg class="ring" width={size} height={size} viewBox="0 0 16 16" role="progressbar" aria-busy="true">
    <circle cx="8" cy="8" r="6.5" />
  </svg>
{/if}

<style>
  .ring {
    animation: spin 2s linear infinite;
    flex-shrink: 0;
  }
  circle {
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.75;
    stroke-linecap: round;
    stroke-dasharray: 1 41;
    transform-origin: center;
    animation: arc 1.6s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  @keyframes arc {
    0% {
      stroke-dasharray: 1 41;
      stroke-dashoffset: 0;
    }
    50% {
      stroke-dasharray: 28 41;
      stroke-dashoffset: -6;
    }
    100% {
      stroke-dasharray: 1 41;
      stroke-dashoffset: -40;
    }
  }

  .spinner {
    flex-shrink: 0;
    /* steps: the spokes jump from one position to the next, like AppKit's */
    animation: spin 0.8s steps(8) infinite;
  }
  line {
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
  }
</style>
