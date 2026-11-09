<script lang="ts">
  import { BookMarked } from "@lucide/svelte";
  import type { Snippet } from "svelte";

  let {
    icon: Icon,
    message,
    children,
  }: {
    icon: typeof BookMarked;
    message: string;
    children?: Snippet;
  } = $props();
</script>

<div class="card flex flex-col items-center justify-center gap-3 py-14 text-center">
  <div class="flex size-12 items-center justify-center rounded-full border border-line bg-bg">
    <span class="text-muted"><Icon class="size-6" /></span>
  </div>
  <p class="max-w-xs text-sm text-muted">{message}</p>
  {#if children}{@render children()}{/if}
</div>
