<script lang="ts">
import { Play } from "@lucide/svelte";
import { openPersona } from "../api";
import { m } from "../paraglide/messages";
import { filterProviders } from "../settings";
import { accounts, activePersona, openOrder, providers } from "../stores";
import type { Category } from "../types";

let tab = $state<"image" | "video">("image");
let selectedAccount = $state("");
let launchingId = $state("");
let error = $state("");

const visible = $derived(filterProviders($providers, tab satisfies Exclude<Category, "both">));

async function launch(providerId: string) {
	const provider = $providers.find((p) => p.id === providerId);
	const account = $accounts.find((a) => a.id === selectedAccount);
	if (!provider || !account || launchingId) return;
	launchingId = providerId;
	error = "";
	try {
		const result = await openPersona(account.id, provider.url, provider.engine);
		activePersona.set(account.id);
		openOrder.set(result.openOrder);
	} catch (e) {
		error = m.errorCommandFailed({ message: String(e) });
	} finally {
		launchingId = "";
	}
}
</script>

<div class="view">
  <h1 class="view-title">{m.menuGenerate()}</h1>

  <div class="flex items-end justify-between gap-3">
    <div class="inline-flex rounded-md border border-line bg-surface p-0.5">
      {#each [["image", m.generateImage()], ["video", m.generateVideo()]] as [id, label]}
        <button
          type="button"
          class="h-8 rounded-sm px-4 text-sm transition-colors {tab === id ? "bg-accent text-accent-fg font-medium" : "hover:bg-bg"}"
          onclick={() => (tab = id as "image" | "video")}
        >
          {label}
        </button>
      {/each}
    </div>
    <label class="flex w-60 flex-col gap-1">
      <span class="view-desc">{m.generateAccount()}</span>
      <select class="input" bind:value={selectedAccount}>
        <option value="" disabled>{m.commonNone()}</option>
        {#each $accounts as a (a.id)}
          <option value={a.id}>{a.name}</option>
        {/each}
      </select>
    </label>
  </div>

  {#if error}<p class="error-text">{error}</p>{/if}

  {#if visible.length === 0}
    <p class="view-desc">{m.generateEmpty()}</p>
  {:else}
    <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3">
      {#each visible as provider (provider.id)}
        <div class="card flex items-center justify-between gap-2 py-3">
          <div class="min-w-0">
            <div class="truncate text-sm font-medium">{provider.name}</div>
            <div class="truncate text-xs text-muted">{provider.url}</div>
          </div>
          <button
            class="btn-icon border border-line"
            aria-label={m.commonLaunch()}
            disabled={!selectedAccount || launchingId === provider.id}
            onclick={() => launch(provider.id)}
          >
            <Play class="size-4" />
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>
