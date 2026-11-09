<script lang="ts">
  import { Lock } from "@lucide/svelte";
  import { isValidProviderUrl, saveProviders } from "../api";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { providers } from "../stores";
  import type { Category, Engine, Provider } from "../types";
  import Modal from "../ui/Modal.svelte";

  let { provider, onclose }: { provider: Provider | null; onclose: () => void } = $props();

  // svelte-ignore state_referenced_locally
  let name = $state(provider?.name ?? "");
  // svelte-ignore state_referenced_locally
  let url = $state(provider?.url ?? "");
  // svelte-ignore state_referenced_locally
  let category = $state<Category>(provider?.category ?? "both");
  // svelte-ignore state_referenced_locally
  let engine = $state<Engine>(provider?.engine ?? "webview");
  let saving = $state(false);
  let error = $state("");

  // Built-in providers: category+engine editable, name/URL locked.
  const locked = $derived(provider !== null && !provider.custom);

  const catLabels = $derived({
    image: m.categoryImage(),
    video: m.categoryVideo(),
    both: m.categoryBoth(),
  });
  const engineLabels = $derived({ webview: m.engineWebview(), chrome: m.engineChrome() });

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    if (saving) return;
    const trimmedName = name.trim();
    if (!locked && !trimmedName) {
      error = m.errorRequired();
      return;
    }
    const trimmedUrl = url.trim();
    if (!locked && !isValidProviderUrl(trimmedUrl)) {
      error = m.errorInvalidUrl();
      return;
    }
    error = "";
    saving = true;
    const next = provider
      ? $providers.map((p) =>
          p.id === provider.id
            ? { ...p, category, engine, ...(locked ? {} : { name: trimmedName, url: trimmedUrl }) }
            : p,
        )
      : [
          ...$providers,
          {
            id: crypto.randomUUID(),
            name: trimmedName,
            url: trimmedUrl,
            category,
            engine,
            custom: true,
          } satisfies Provider,
        ];
    try {
      await saveProviders(next);
      providers.set(next);
      onclose();
    } catch (err) {
      error = commandError(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={provider ? m.commonEdit() : m.commonAdd()} {onclose}>
  <form class="space-y-3" onsubmit={submit}>
    <label class="flex flex-col gap-1">
      <span class="flex items-center gap-1 view-desc">
        {m.commonName()}
        {#if locked}<Lock class="size-3" />{/if}
      </span>
      <input class="input" bind:value={name} disabled={locked} required />
    </label>
    <label class="flex flex-col gap-1">
      <span class="flex items-center gap-1 view-desc">
        {m.commonUrl()}
        {#if locked}<Lock class="size-3" />{/if}
      </span>
      <input class="input" bind:value={url} disabled={locked} placeholder="https://…" required />
    </label>
    {#if locked}
      <p class="view-desc">{m.providersLockedHint()}</p>
    {/if}
    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1">
        <span class="view-desc">{m.commonCategory()}</span>
        <select class="input" bind:value={category}>
          {#each ["image", "video", "both"] as c (c)}
            <option value={c}>{catLabels[c as Category]}</option>
          {/each}
        </select>
      </label>
      <label class="flex flex-col gap-1">
        <span class="view-desc">{m.commonEngine()}</span>
        <select class="input" bind:value={engine}>
          {#each ["webview", "chrome"] as en (en)}
            <option value={en}>{engineLabels[en as Engine]}</option>
          {/each}
        </select>
      </label>
    </div>
    {#if error}<p class="error-text">{error}</p>{/if}
    <div class="flex gap-2">
      <button class="btn-primary" type="submit" disabled={saving}>{m.commonSave()}</button>
      <button class="btn" type="button" onclick={onclose}>{m.commonCancel()}</button>
    </div>
  </form>
</Modal>
