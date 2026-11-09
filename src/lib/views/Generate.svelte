<script lang="ts">
  import { Play, Sparkles } from "@lucide/svelte";
  import { commandError } from "../errors";
  import { launchPersona } from "../launch";
  import { m } from "../paraglide/messages";
  import { copyText } from "../clipboard";
  import { filterProviders } from "../settings";
  import { accounts, prompts, providers } from "../stores";
  import { toast } from "../toast";
  import EmptyState from "../ui/EmptyState.svelte";
  import Select from "../ui/Select.svelte";

  let tab = $state<"image" | "video">("image");
  let selectedProviderId = $state("");
  let selectedAccountId = $state("");
  let selectedPromptId = $state("");
  let launching = $state(false);

  const engineLabels = $derived({ webview: m.engineWebview(), chrome: m.engineChrome() });
  const visible = $derived(filterProviders($providers, tab));
  const selectedProvider = $derived($providers.find((p) => p.id === selectedProviderId) ?? null);
  const selectedAccount = $derived($accounts.find((a) => a.id === selectedAccountId) ?? null);
  const selectedPrompt = $derived($prompts.find((p) => p.id === selectedPromptId) ?? null);

  // Pinned prompts first.
  const sortedPrompts = $derived([...$prompts].sort((a, b) => Number(b.pinned) - Number(a.pinned)));

  function selectProvider(id: string) {
    selectedProviderId = id;
  }

  async function go() {
    const provider = selectedProvider;
    const account = selectedAccount;
    const prompt = selectedPrompt;
    if (!provider || !account || launching) return;
    launching = true;
    try {
      await launchPersona(account, provider);
      if (prompt) {
        await copyText(prompt.body);
        toast(m.generatePromptCopied());
      }
    } catch (e) {
      toast(commandError(e));
    } finally {
      launching = false;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.ctrlKey && !e.altKey && e.key === "Enter") {
      e.preventDefault();
      void go();
    }
  }
</script>

<svelte:window onkeydown={onkeydown} />

<div class="view">
  <div class="space-y-1">
    <h1 class="view-title">{m.menuGenerate()}</h1>
    <p class="view-desc">{m.generateDescription()}</p>
  </div>

  <div class="flex w-fit gap-1.5">
    {#each [["image", m.generateImage()], ["video", m.generateVideo()]] as [id, label] (id)}
      <button
        type="button"
        class="h-7 rounded-md border px-2.5 text-xs transition-colors duration-150 {tab === id
          ? "border-accent bg-accent font-medium text-accent-fg"
          : "border-line bg-surface text-muted hover:bg-bg hover:text-fg"}"
        aria-pressed={tab === id}
        onclick={() => {
          tab = id as "image" | "video";
          selectedProviderId = "";
        }}
      >
        {label}
      </button>
    {/each}
  </div>

  {#if visible.length === 0}
    <EmptyState icon={Sparkles} message={m.generateEmpty()} />
  {:else}
    <div
      class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-1.5"
      role="group"
      aria-label={m.menuProviders()}
    >
      {#each visible as provider (provider.id)}
        <button
          type="button"
          class="flex min-h-[76px] flex-col justify-between gap-2 rounded-md border p-3 text-left transition-colors duration-150 {selectedProviderId === provider.id
            ? "border-accent bg-accent/10"
            : "border-line bg-surface hover:bg-bg"}"
          aria-pressed={selectedProviderId === provider.id}
          onclick={() => selectProvider(provider.id)}
        >
          <span class="flex w-full items-center justify-between gap-2">
            <span
              class="flex size-7 items-center justify-center rounded-md border border-line bg-bg text-xs font-semibold {selectedProviderId === provider.id
                ? "text-accent"
                : "text-muted"}"
              aria-hidden="true"
            >
              {provider.name.charAt(0).toUpperCase()}
            </span>
            <span class="chip">{engineLabels[provider.engine]}</span>
          </span>
          <span class="truncate text-sm font-medium">{provider.name}</span>
        </button>
      {/each}
    </div>

    {#if selectedProvider}
      <div class="card space-y-3">
        <div class="flex items-center gap-2">
          <span
            class="flex size-7 items-center justify-center rounded-md border border-line bg-bg text-xs font-semibold text-accent"
            aria-hidden="true"
          >
            {selectedProvider.name.charAt(0).toUpperCase()}
          </span>
          <span class="text-lg font-semibold">{selectedProvider.name}</span>
          <span class="chip">{engineLabels[selectedProvider.engine]}</span>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1">
            <span class="view-desc">{m.menuAccounts()}</span>
            <Select
              ariaLabel={m.menuAccounts()}
              bind:value={selectedAccountId}
              options={[
                { value: "", label: m.commonNone(), disabled: true },
                ...$accounts.map((a) => ({ value: a.id, label: a.name })),
              ]}
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="view-desc">{m.menuPrompts()}</span>
            <Select
              ariaLabel={m.menuPrompts()}
              bind:value={selectedPromptId}
              options={[
                { value: "", label: m.commonNone() },
                ...sortedPrompts.map((p) => ({ value: p.id, label: p.title })),
              ]}
            />
          </label>
        </div>
        <div class="flex justify-end">
          <button
            class="btn-primary h-10 px-6 text-base"
            disabled={!selectedAccount || launching}
            onclick={() => void go()}
          >
            <Play class="size-4" />
            {selectedPrompt ? m.generateOpenCopy() : m.commonLaunch()}
          </button>
        </div>
      </div>
    {/if}
  {/if}
</div>
