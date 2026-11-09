<script lang="ts">
  import { Check, ChevronDown } from "@lucide/svelte";

  type Props = {
    options: SelectOption[];
    /** Accessible name for the trigger; the visible label is a sibling span. */
    ariaLabel: string;
    /** Optional two-way binding; fires before `onchange`. */
    value?: string;
    onchange?: (value: string) => void;
    id?: string;
  };

  let { options, ariaLabel, value = $bindable(""), onchange, id }: Props = $props();

  let open = $state(false);
  let active = $state(0);
  let root: HTMLDivElement | undefined = $state();
  let list: HTMLUListElement | undefined = $state();

  const selectedLabel = $derived(options.find((o) => o.value === value)?.label ?? value);

  function choose(option: SelectOption) {
    if (option.disabled) return;
    value = option.value;
    onchange?.(option.value);
    open = false;
  }

  function syncActive() {
    const i = options.findIndex((o) => o.value === value && !o.disabled);
    active = i >= 0 ? i : options.findIndex((o) => !o.disabled);
  }

  function move(delta: number) {
    for (let i = active + delta; i >= 0 && i < options.length; i += delta) {
      if (!options[i].disabled) {
        active = i;
        break;
      }
    }
    list?.querySelectorAll("button")[active]?.scrollIntoView({ block: "nearest" });
  }

  function onkeydown(e: KeyboardEvent) {
    if (!open && ["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
      e.preventDefault();
      syncActive();
      open = true;
    } else if (open && e.key === "ArrowDown") {
      e.preventDefault();
      move(1);
    } else if (open && e.key === "ArrowUp") {
      e.preventDefault();
      move(-1);
    } else if (open && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      choose(options[active]);
    } else if (open && e.key === "Escape") {
      open = false;
    }
  }

  function onoutside(e: MouseEvent) {
    if (open && root && !root.contains(e.target as Node)) open = false;
  }
</script>

<!-- ponytail: no type-ahead search in the popup; add only if lists grow past ~30 items. -->
<svelte:window onclick={onoutside} />

<div class="relative" bind:this={root}>
  <button
    {id}
    type="button"
    class="input flex items-center justify-between gap-2 text-left"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={ariaLabel}
    onclick={() => {
      if (open) {
        open = false;
      } else {
        syncActive();
        open = true;
      }
    }}
    onkeydown={onkeydown}
  >
    <span class="truncate">{selectedLabel}</span>
    <ChevronDown
      class="size-4 shrink-0 text-muted transition-transform duration-150 {open ? 'rotate-180' : ''}"
    />
  </button>
  {#if open}
    <ul
      role="listbox"
      class="absolute top-full left-0 z-20 mt-1 max-h-60 w-full overflow-y-auto rounded-md border border-line bg-surface py-1 shadow-lg"
      bind:this={list}
    >
      {#each options as option, i (option.value)}
        <li role="presentation">
          <button
            type="button"
            role="option"
            aria-selected={option.value === value}
            disabled={option.disabled}
            class="flex w-full items-center justify-between gap-2 px-2.5 py-1.5 text-left text-sm transition-colors duration-150 {option.disabled
              ? "cursor-default text-muted opacity-60"
              : option.value === value
                ? "bg-bg font-medium text-accent"
                : i === active
                  ? "bg-bg text-fg"
                  : "text-fg hover:bg-bg"}"
            onmouseenter={() => (active = i)}
            onclick={() => choose(option)}
          >
            <span class="truncate">{option.label}</span>
            {#if option.value === value}<Check class="size-3.5 shrink-0" />{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<script module lang="ts">
  /** One choice in a Select dropdown. */
  export type SelectOption = { value: string; label: string; disabled?: boolean };
</script>
