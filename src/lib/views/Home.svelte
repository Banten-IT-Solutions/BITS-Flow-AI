<script lang="ts">
import { Play } from "@lucide/svelte";
import { openPersona } from "../api";
import { m } from "../paraglide/messages";
import { accounts, activePersona, openOrder, providers } from "../stores";

let selectedProvider = $state("");
let selectedAccount = $state("");
let launching = $state(false);
let error = $state("");

async function launch() {
	const provider = $providers.find((p) => p.id === selectedProvider);
	const account = $accounts.find((a) => a.id === selectedAccount);
	if (!provider || !account || launching) return;
	launching = true;
	error = "";
	try {
		const result = await openPersona(account.id, provider.url, provider.engine);
		activePersona.set(account.id);
		openOrder.set(result.openOrder);
	} catch (e) {
		error = m.errorCommandFailed({ message: String(e) });
	} finally {
		launching = false;
	}
}
</script>

<div class="view">
  <h1 class="view-title">{m.homeTitle()}</h1>

  <section class="card space-y-3">
    <h2 class="text-sm font-medium">{m.homeQuickLaunch()}</h2>
    <p class="view-desc">{m.homeQuickLaunchHint()}</p>
    <div class="flex flex-wrap items-end gap-3">
      <label class="flex w-60 flex-col gap-1">
        <span class="view-desc">{m.menuProviders()}</span>
        <select class="input" bind:value={selectedProvider}>
          <option value="" disabled>{m.commonNone()}</option>
          {#each $providers as p (p.id)}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
      </label>
      <label class="flex w-60 flex-col gap-1">
        <span class="view-desc">{m.menuAccounts()}</span>
        <select class="input" bind:value={selectedAccount}>
          <option value="" disabled>{m.commonNone()}</option>
          {#each $accounts as a (a.id)}
            <option value={a.id}>{a.name}</option>
          {/each}
        </select>
      </label>
      <button class="btn-primary" disabled={!selectedProvider || !selectedAccount || launching} onclick={launch}>
        <Play class="size-4" />
        {launching ? "…" : m.commonLaunch()}
      </button>
    </div>
    {#if error}<p class="error-text">{error}</p>{/if}
  </section>

  <section class="space-y-2">
    <h2 class="text-sm font-medium">{m.homePersonas()}</h2>
    {#if $accounts.length === 0}
      <p class="view-desc">{m.homeNoPersonas()}</p>
    {:else}
      <div class="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3">
        {#each $accounts as account (account.id)}
          <div class="flex items-center gap-3 rounded-md border border-line bg-surface px-3 py-2.5">
            <span
              class="size-3 shrink-0 rounded-full border border-line"
              style:background={account.color ?? "var(--fg-muted)"}
            ></span>
            <div class="min-w-0 flex-1">
              <div class="truncate text-sm font-medium">{account.name}</div>
              <div class="truncate text-xs text-muted">{account.email}</div>
            </div>
            {#if $activePersona === account.id}
              <span class="badge">{m.homeActiveBadge()}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>
