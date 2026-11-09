<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { toasts } from "../toast";

  function run(id: number, run: () => void) {
    run();
    toasts.update((list) => list.filter((t) => t.id !== id));
  }
</script>

<div class="fixed right-4 bottom-4 z-[60] flex w-80 flex-col gap-2" role="status" aria-live="polite">
  {#each $toasts as t (t.id)}
    <div
      class="flex items-center gap-2 rounded-md border border-line bg-surface px-3 py-2 text-sm"
      in:fly={{ y: 8, duration: 150 }}
      out:fade={{ duration: 150 }}
    >
      <span class="min-w-0 flex-1">{t.message}</span>
      {#if t.action}
        <button
          type="button"
          class="shrink-0 text-sm font-medium text-accent transition-opacity duration-150 hover:opacity-80"
          onclick={() => run(t.id, t.action!.run)}
        >
          {t.action.label}
        </button>
      {/if}
    </div>
  {/each}
</div>
