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
  <h1 class="view-title">{m.menuGenerate()}</h1>

  <div class="inline-flex w-fit rounded-md border border-line bg-surface p-0.5">
    {#each [["image", m.generateImage()], ["video", m.generateVideo()]] as [id, label] (id)}
      <button
        type="button"
        class="h-8 rounded-sm px-4 text-sm transition-colors duration-150 {tab === id
          ? "bg-accent font-medium text-accent-fg"
          : "text-muted hover:bg-bg hover:text-fg"}"
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
    <ul class="space-y-1.5">
      {#each visible as provider (provider.id)}
        <li>
          <div
            class="row cursor-pointer pr-1.5 transition-colors duration-150 {selectedProviderId === provider.id
              ? "border-accent"
              : ""}"
            role="button"
            tabindex="0"
            onclick={() => selectProvider(provider.id)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                selectProvider(provider.id);
              }
            }}
          >
            <span class="min-w-0 flex-1 truncate text-sm">{provider.name}</span>
            <span class="chip shrink-0">{engineLabels[provider.engine]}</span>
            <button
              type="button"
              class="btn h-8"
              disabled={!selectedAccount || launching}
              onclick={(e) => {
                e.stopPropagation();
                selectProvider(provider.id);
                void go();
              }}
            >
              <Play class="size-3.5" />
              {m.commonLaunch()}
            </button>
          </div>
          {#if selectedProviderId === provider.id}
            <div class="card mt-1.5 space-y-3">
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
              <button
                class="btn-primary h-10 px-6 text-base"
                disabled={!selectedAccount || launching}
                onclick={() => void go()}
              >
                <Play class="size-4" />
                {selectedPrompt ? m.generateOpenCopy() : m.commonLaunch()}
              </button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>
