<!-- WinUI ProgressRing (indeterminate): a rotating arc that grows and shrinks. -->
<script lang="ts">
  let { size = 16 }: { size?: number } = $props();
</script>

<svg class="ring" width={size} height={size} viewBox="0 0 16 16" role="progressbar" aria-busy="true">
  <circle cx="8" cy="8" r="6.5" />
</svg>

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
</style>
