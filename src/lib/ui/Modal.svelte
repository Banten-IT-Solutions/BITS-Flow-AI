<script lang="ts">
  import { X } from "@lucide/svelte";
  import { fade, fly } from "svelte/transition";
  import { m } from "../paraglide/messages";

  let {
    title,
    onclose,
    children,
  }: {
    title: string;
    onclose: () => void;
    children: import("svelte").Snippet;
  } = $props();

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window onkeydown={onkeydown} />

<div
  class="fixed inset-0 z-40 bg-black/40"
  transition:fade={{ duration: 150 }}
  onclick={onclose}
  aria-hidden="true"
></div>
<div class="pointer-events-none fixed inset-0 z-50 flex items-center justify-center p-6">
  <div
    class="pointer-events-auto w-full max-w-md rounded-lg border border-line bg-surface"
    transition:fly={{ y: 8, duration: 150 }}
    role="dialog"
    aria-modal="true"
    aria-label={title}
  >
    <header class="flex h-12 items-center justify-between border-b border-line px-4">
      <h2 class="text-base font-semibold">{title}</h2>
      <button class="btn-icon" aria-label={m.commonClose()} onclick={onclose}>
        <X class="size-4" />
      </button>
    </header>
    <div class="p-4">
      {@render children()}
    </div>
  </div>
</div>
