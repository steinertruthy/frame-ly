<script lang="ts">
  // Click-through border around a region being recorded (excluded from the recording).
</script>

<div class="frame"></div>

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
  }
  .frame {
    position: fixed;
    inset: 1px;
    border: 2px dashed rgba(217, 70, 86, 0.9);
    border-radius: 4px;
    animation: breathe 2.4s ease-in-out infinite;
  }
  @keyframes breathe {
    50% {
      border-color: rgba(217, 70, 86, 0.45);
    }
  }
</style>
