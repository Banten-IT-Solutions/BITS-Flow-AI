<script module lang="ts">
  import { EllipsisVertical } from "@lucide/svelte";

  export interface MenuAction {
    kind: "action";
    label: string;
    icon?: typeof EllipsisVertical;
    danger?: boolean;
    disabled?: boolean;
    onselect: () => void;
  }

  /** One-level submenu: header + flat action list (e.g. "Open in…" providers). */
  export interface MenuGroup {
    kind: "group";
    label: string;
    items: { label: string; onselect: () => void }[];
  }

  export type MenuItem = MenuAction | MenuGroup;
</script>

<script lang="ts">
  import { fly } from "svelte/transition";

  let { items, label }: { items: MenuItem[]; label: string } = $props();

  let open = $state(false);
  let expanded = $state("");
  let root: HTMLElement | undefined = $state();

  function select(action: () => void) {
    open = false;
    expanded = "";
    action();
  }

  function onwindowpointerdown(e: PointerEvent) {
    if (open && root && !root.contains(e.target as Node)) {
      open = false;
      expanded = "";
    }
  }

  function onwindowkeydown(e: KeyboardEvent) {
    if (open && e.key === "Escape") {
      e.stopPropagation();
      open = false;
      expanded = "";
    }
  }
</script>

<svelte:window onpointerdown={onwindowpointerdown} onkeydown={onwindowkeydown} />

<div bind:this={root} class="relative shrink-0">
  <button
    type="button"
    class="btn-icon"
    aria-label={label}
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
    <EllipsisVertical class="size-4" />
  </button>

  {#if open}
    <div
      class="absolute top-full right-0 z-30 mt-1 min-w-44 rounded-md border border-line bg-surface py-1"
      transition:fly={{ y: 4, duration: 150 }}
      role="menu"
    >
      {#each items as item (item.kind === "action" ? item.label : item.label)}
        {#if item.kind === "action"}
          <button
            type="button"
            role="menuitem"
            class="flex h-8 w-full items-center gap-2 px-3 text-left text-sm transition-colors duration-150 hover:bg-bg {item.danger
              ? "text-danger"
              : ""}"
            disabled={item.disabled}
            onclick={() => select(item.onselect)}
          >
            {#if item.icon}
              <item.icon class="size-4 shrink-0" />
            {/if}
            {item.label}
          </button>
        {:else}
          <button
            type="button"
            class="flex h-8 w-full items-center gap-2 px-3 text-left text-sm transition-colors duration-150 hover:bg-bg"
            onclick={() => (expanded = expanded === item.label ? "" : item.label)}
          >
            {item.label}
          </button>
          {#if expanded === item.label}
            <div class="max-h-44 overflow-y-auto border-t border-line">
              {#each item.items as sub (sub.label)}
                <button
                  type="button"
                  role="menuitem"
                  class="flex h-8 w-full items-center gap-2 pr-3 pl-6 text-left text-sm transition-colors duration-150 hover:bg-bg"
                  onclick={() => select(sub.onselect)}
                >
                  {sub.label}
                </button>
              {/each}
            </div>
          {/if}
        {/if}
      {/each}
    </div>
  {/if}
</div>
