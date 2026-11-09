<script lang="ts">
import { Pencil, Plus, Trash2, X } from "@lucide/svelte";
import { isValidProviderUrl, saveProviders } from "../api";
import { m } from "../paraglide/messages";
import { categoryLabel } from "../settings";
import { providers } from "../stores";
import type { Category, Provider } from "../types";

let adding = $state(false);
let editingId = $state("");
let draft = $state({ name: "", url: "", category: "both" as Category });
let saving = $state(false);
let error = $state("");

const catLabels = $derived({
	image: m.categoryImage(),
	video: m.categoryVideo(),
	both: m.categoryBoth(),
});
const engineLabels = $derived({ webview: m.engineWebview(), chrome: m.engineChrome() });

function startAdd() {
	editingId = "";
	draft = { name: "", url: "", category: "both" };
	adding = true;
	error = "";
}

function startEdit(provider: Provider) {
	adding = false;
	editingId = provider.id;
	draft = { name: provider.name, url: provider.url, category: provider.category };
	error = "";
}

function cancel() {
	adding = false;
	editingId = "";
	error = "";
}

function validate(): boolean {
	if (!draft.name.trim()) {
		error = m.errorRequired();
		return false;
	}
	if (!isValidProviderUrl(draft.url.trim())) {
		error = m.errorInvalidUrl();
		return false;
	}
	error = "";
	return true;
}

async function persist(next: Provider[]) {
	saving = true;
	try {
		await saveProviders(next);
		providers.set(next);
		cancel();
	} catch (e) {
		error = m.errorCommandFailed({ message: String(e) });
	} finally {
		saving = false;
	}
}

function submit() {
	if (!validate() || saving) return;
	if (editingId) {
		const next = $providers.map((p) =>
			p.id === editingId
				? { ...p, name: draft.name.trim(), url: draft.url.trim(), category: draft.category }
				: p,
		);
		void persist(next);
	} else {
		const provider: Provider = {
			id: crypto.randomUUID(),
			name: draft.name.trim(),
			url: draft.url.trim(),
			category: draft.category,
			engine: "webview",
			custom: true,
		};
		void persist([...$providers, provider]);
	}
}

function remove(provider: Provider) {
	if (!window.confirm(m.commonConfirmDelete({ name: provider.name }))) return;
	void persist($providers.filter((p) => p.id !== provider.id));
}

const showForm = $derived(adding || editingId !== "");
</script>

<div class="view">
  <div class="flex items-center justify-between">
    <div class="space-y-1">
      <h1 class="view-title">{m.providersTitle()}</h1>
      <p class="view-desc">{m.providersDescription()}</p>
    </div>
    {#if !showForm}
      <button class="btn-primary" onclick={startAdd}>
        <Plus class="size-4" />
        {m.commonAdd()}
      </button>
    {/if}
  </div>

  {#if showForm}
    <form class="card space-y-3" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <div class="grid grid-cols-[1fr_2fr_auto_auto_auto] items-end gap-3">
        <label class="flex flex-col gap-1">
          <span class="view-desc">{m.commonName()}</span>
          <input class="input" bind:value={draft.name} required />
        </label>
        <label class="flex flex-col gap-1">
          <span class="view-desc">{m.commonUrl()}</span>
          <input class="input" bind:value={draft.url} placeholder="https://…" required />
        </label>
        <label class="flex flex-col gap-1">
          <span class="view-desc">{m.commonCategory()}</span>
          <select class="input" bind:value={draft.category}>
            <option value="image">{catLabels.image}</option>
            <option value="video">{catLabels.video}</option>
            <option value="both">{catLabels.both}</option>
          </select>
        </label>
        <button class="btn-primary" type="submit" disabled={saving}>{m.commonSave()}</button>
        <button class="btn" type="button" onclick={cancel}>
          <X class="size-4" />
        </button>
      </div>
      {#if error}<p class="error-text">{error}</p>{/if}
    </form>
  {/if}

  {#if $providers.length === 0}
    <p class="view-desc">{m.providersEmpty()}</p>
  {:else}
    <ul class="space-y-1.5">
      {#each $providers as provider (provider.id)}
        <li class="row py-1.5">
          <span class="min-w-0 flex-1 truncate text-sm">{provider.name}</span>
          <span class="min-w-0 flex-[2] truncate text-xs text-muted">{provider.url}</span>
          <span class="badge {provider.custom ? "" : "opacity-80"}">
            {provider.custom ? m.providersCustom() : m.providersBuiltin()}
          </span>
          <span class="w-28 truncate text-xs text-muted">{categoryLabel(provider.category, catLabels)}</span>
          <span class="w-32 truncate text-xs text-muted">{engineLabels[provider.engine]}</span>
          <button class="btn-icon" aria-label={m.commonEdit()} onclick={() => startEdit(provider)}>
            <Pencil class="size-4" />
          </button>
          <button class="btn-icon hover:text-danger" aria-label={m.commonDelete()} onclick={() => remove(provider)}>
            <Trash2 class="size-4" />
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
