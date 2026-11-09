<script lang="ts">
  import { Lock, Plus } from "@lucide/svelte";
  import { saveProviders } from "../api";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { categoryLabel } from "../settings";
  import { providers } from "../stores";
  import { toast } from "../toast";
  import type { Category, Provider } from "../types";
  import { get } from "svelte/store";
  import ConfirmDelete from "../ui/ConfirmDelete.svelte";
  import Menu from "../ui/Menu.svelte";
  import ProviderDialog from "../ui/ProviderDialog.svelte";

  type Filter = "all" | Category;

  let query = $state("");
  let filter = $state<Filter>("all");
  let dialogProvider = $state<Provider | null>(null);
  let dialogOpen = $state(false);
  let confirmTarget = $state<Provider | null>(null);
  let confirmBusy = $state(false);
  let confirmError = $state("");

  const catLabels = $derived({
    image: m.categoryImage(),
    video: m.categoryVideo(),
    both: m.categoryBoth(),
  });
  const engineLabels = $derived({ webview: m.engineWebview(), chrome: m.engineChrome() });

  const filters: { id: Filter; label: () => string }[] = [
    { id: "all", label: () => m.providersFilterAll() },
    { id: "image", label: () => catLabels.image },
    { id: "video", label: () => catLabels.video },
    { id: "both", label: () => catLabels.both },
  ];

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return $providers.filter(
      (p) =>
        (filter === "all" || p.category === filter) &&
        (!q || p.name.toLowerCase().includes(q) || p.url.toLowerCase().includes(q)),
    );
  });

  async function remove(provider: Provider) {
    if (confirmBusy) return;
    confirmBusy = true;
    confirmError = "";
    const idx = $providers.findIndex((p) => p.id === provider.id);
    const next = $providers.filter((p) => p.id !== provider.id);
    try {
      await saveProviders(next);
      providers.set(next);
      confirmTarget = null;
      toast(m.commonDeletedToast({ name: provider.name }), {
        label: m.commonUndo(),
        run: () => void restore(provider, idx),
      });
    } catch (e) {
      confirmError = commandError(e);
    } finally {
      confirmBusy = false;
    }
  }

  async function restore(provider: Provider, idx: number) {
    const list = [...get(providers)];
    list.splice(Math.min(idx, list.length), 0, provider);
    try {
      await saveProviders(list);
      providers.set(list);
    } catch (e) {
      toast(commandError(e));
    }
  }

  function tileLetter(provider: Provider): string {
    return (provider.name.trim()[0] ?? "?").toUpperCase();
  }
</script>

<div class="view">
  <div class="flex items-center justify-between gap-3">
    <div class="space-y-1">
      <h1 class="view-title">{m.providersTitle()}</h1>
      <p class="view-desc">{m.providersDescription()}</p>
    </div>
    <div class="flex items-center gap-2">
      <input
        class="input w-56"
        data-search-input
        placeholder={m.commonSearch()}
        bind:value={query}
      />
      <button
        class="btn-primary"
        onclick={() => {
          dialogProvider = null;
          dialogOpen = true;
        }}
      >
        <Plus class="size-4" />
        {m.commonAdd()}
      </button>
    </div>
  </div>

  <div class="flex flex-wrap items-center gap-1.5">
    {#each filters as f (f.id)}
      <button
        type="button"
        class="h-7 rounded-md border px-2.5 text-xs transition-colors duration-150 {filter === f.id
          ? "border-accent bg-accent font-medium text-accent-fg"
          : "border-line bg-surface text-muted hover:bg-bg hover:text-fg"}"
        aria-pressed={filter === f.id}
        onclick={() => (filter = f.id)}
      >
        {f.label()}
      </button>
    {/each}
  </div>

  {#if $providers.length === 0}
    <p class="view-desc">{m.providersEmpty()}</p>
  {:else if filtered.length === 0}
    <p class="view-desc">{m.commonNoResults()}</p>
  {:else}
    <ul class="space-y-1.5">
      {#each filtered as provider (provider.id)}
        <li class="row pr-1.5">
          <span
            class="flex size-8 shrink-0 items-center justify-center rounded-md border border-line bg-bg text-sm font-semibold text-muted"
            aria-hidden="true"
          >
            {tileLetter(provider)}
          </span>
          <span class="min-w-0 flex-1 truncate text-sm">
            {provider.name}
            {#if !provider.custom}<Lock class="ml-1 inline size-3 align-[-2px] text-muted" />{/if}
          </span>
          <span class="min-w-0 flex-[2] truncate text-xs text-muted">{provider.url}</span>
          <span class="chip shrink-0">{categoryLabel(provider.category, catLabels)}</span>
          <span class="chip shrink-0">{engineLabels[provider.engine]}</span>
          <Menu
            label={m.commonActions()}
            items={[
              {
                kind: "action" as const,
                label: m.commonEdit(),
                onselect: () => {
                  dialogProvider = provider;
                  dialogOpen = true;
                },
              },
              ...(provider.custom
                ? [
                    {
                      kind: "action" as const,
                      label: m.commonDelete(),
                      danger: true,
                      onselect: () => {
                        confirmTarget = provider;
                        confirmError = "";
                      },
                    },
                  ]
                : []),
            ]}
          />
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if dialogOpen}
  <ProviderDialog provider={dialogProvider} onclose={() => (dialogOpen = false)} />
{/if}

{#if confirmTarget}
  <ConfirmDelete
    name={confirmTarget.name}
    title={m.commonDelete()}
    busy={confirmBusy}
    error={confirmError}
    onconfirm={() => void remove(confirmTarget!)}
    oncancel={() => (confirmTarget = null)}
  />
{/if}
