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
  let launching = $state(false);
  let accountByProvider = $state<Record<string, string>>({});
  let promptByProvider = $state<Record<string, string>>({});

  const engineLabels = $derived({ webview: m.engineWebview(), chrome: m.engineChrome() });
  const visible = $derived(filterProviders($providers, tab));

  // Pinned prompts first.
  const sortedPrompts = $derived([...$prompts].sort((a, b) => Number(b.pinned) - Number(a.pinned)));

  async function go(providerId: string) {
    const provider = $providers.find((p) => p.id === providerId);
    const account = $accounts.find((a) => a.id === accountByProvider[providerId]);
    const prompt = $prompts.find((p) => p.id === promptByProvider[providerId]);
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
</script>

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
        onclick={() => (tab = id as "image" | "video")}
      >
        {label}
      </button>
    {/each}
  </div>

  {#if visible.length === 0}
    <EmptyState icon={Sparkles} message={m.generateEmpty()} />
  {:else}
    <div
      class="grid grid-cols-[repeat(auto-fill,minmax(230px,1fr))] gap-1.5"
      role="group"
      aria-label={m.menuProviders()}
    >
      {#each visible as provider (provider.id)}
        <div
          class="flex flex-col gap-2 rounded-md border border-line bg-surface p-3 transition-colors duration-150 hover:border-muted/50"
        >
          <div class="flex items-center gap-2">
            <span
              class="flex size-6 shrink-0 items-center justify-center rounded-md border border-line bg-bg text-xs font-semibold text-muted"
              aria-hidden="true"
            >
              {provider.name.charAt(0).toUpperCase()}
            </span>
            <span class="min-w-0 flex-1 truncate text-sm font-medium">{provider.name}</span>
            <span class="chip">{engineLabels[provider.engine]}</span>
          </div>
          <Select
            compact
            ariaLabel={`${m.menuAccounts()} — ${provider.name}`}
            value={accountByProvider[provider.id] ?? ""}
            onchange={(v) => (accountByProvider[provider.id] = v)}
            options={[
              { value: "", label: m.commonNone(), disabled: true },
              ...$accounts.map((a) => ({ value: a.id, label: a.name })),
            ]}
          />
          {#if sortedPrompts.length > 0}
            <Select
              compact
              ariaLabel={`${m.menuPrompts()} — ${provider.name}`}
              value={promptByProvider[provider.id] ?? ""}
              onchange={(v) => (promptByProvider[provider.id] = v)}
              options={[
                { value: "", label: m.commonNone() },
                ...sortedPrompts.map((p) => ({ value: p.id, label: p.title })),
              ]}
            />
          {/if}
          <button
            type="button"
            class="btn-primary h-8 w-full justify-center"
            disabled={!accountByProvider[provider.id] || launching}
            onclick={() => void go(provider.id)}
          >
            <Play class="size-3.5" />
            {promptByProvider[provider.id] ? m.generateOpenCopy() : m.commonLaunch()}
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>
