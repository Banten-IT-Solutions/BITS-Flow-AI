<script lang="ts">
  import { Clapperboard, Globe, Image as ImageIcon, Layers, Play } from "@lucide/svelte";
  import { clearPersonaData, saveAccounts } from "../api";
  import { commandError } from "../errors";
  import { lastUsedProvider, launchPersona, refreshOpenPersonas } from "../launch";
  import { m } from "../paraglide/messages";
  import { accounts, openLabels, providers, recent, uiLocale } from "../stores";
  import { toast } from "../toast";
  import { formatRelative } from "../time";
  import type { Account } from "../types";
  import AccountDialog from "../ui/AccountDialog.svelte";

  import ConfirmDelete from "../ui/ConfirmDelete.svelte";
  import Menu, { type MenuItem } from "../ui/Menu.svelte";
  import Select from "../ui/Select.svelte";

  let selectedProvider = $state("");
  let selectedAccount = $state("");
  let launching = $state(false);

  let editAccount = $state<Account | null>(null);
  let editOpen = $state(false);
  let confirmTarget = $state<Account | null>(null);
  let confirmMode = $state<"delete" | "clear">("delete");
  let confirmBusy = $state(false);
  let confirmError = $state("");

  const catIcons = {
    image: ImageIcon,
    video: Clapperboard,
    both: Layers,
  };

  $effect(() => {
    void refreshOpenPersonas();
  });

  function categoryIcon(providerId: string): typeof Globe {
    const category = $providers.find((p) => p.id === providerId)?.category;
    return category ? catIcons[category] : Globe;
  }

  async function launch(providerId: string, accountId: string) {
    const provider = $providers.find((p) => p.id === providerId);
    const account = $accounts.find((a) => a.id === accountId);
    if (!provider || !account || launching) return;
    launching = true;
    try {
      await launchPersona(account, provider);
    } catch (e) {
      toast(commandError(e));
    } finally {
      launching = false;
    }
  }

  /** Card click / Ctrl+1..9 style open: last-used provider, else first. */
  async function openWithLast(account: Account) {
    const provider = lastUsedProvider(account, $recent, $providers) ?? $providers[0];
    if (!provider) return;
    await launch(provider.id, account.id);
  }

  function personaMenu(account: Account): MenuItem[] {
    return [
      {
        kind: "action",
        label: m.commonEdit(),
        onselect: () => {
          editAccount = account;
          editOpen = true;
        },
      },
      {
        kind: "group",
        label: m.commonOpenIn(),
        items: $providers.map((p) => ({
          label: p.name,
          onselect: () => void launch(p.id, account.id),
        })),
      },
      {
        kind: "action",
        label: m.commonClearData(),
        onselect: () => {
          confirmMode = "clear";
          confirmTarget = account;
          confirmError = "";
        },
      },
      {
        kind: "action",
        label: m.commonDelete(),
        danger: true,
        onselect: () => {
          confirmMode = "delete";
          confirmTarget = account;
          confirmError = "";
        },
      },
    ];
  }

  async function onConfirm() {
    const account = confirmTarget;
    if (!account || confirmBusy) return;
    confirmBusy = true;
    confirmError = "";
    try {
      if (confirmMode === "clear") {
        await clearPersonaData(account.id);
        toast(m.commonDataCleared());
      } else {
        const idx = $accounts.findIndex((a) => a.id === account.id);
        const next = $accounts.filter((a) => a.id !== account.id);
        await saveAccounts(next);
        accounts.set(next);
        toast(m.commonDeletedToast({ name: account.name }), {
          label: m.commonUndo(),
          run: () => void restore(account, idx),
        });
      }
      confirmTarget = null;
    } catch (e) {
      confirmError = commandError(e);
    } finally {
      confirmBusy = false;
    }
  }

  async function restore(account: Account, idx: number) {
    const list = [...$accounts];
    list.splice(Math.min(idx, list.length), 0, account);
    try {
      await saveAccounts(list);
      accounts.set(list);
    } catch (e) {
      toast(commandError(e));
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.ctrlKey && !e.altKey && e.key === "Enter") {
      e.preventDefault();
      void launch(selectedProvider, selectedAccount);
    }
  }
</script>

<svelte:window onkeydown={onkeydown} />

<div class="view">
  <h1 class="view-title">{m.homeTitle()}</h1>

  <section class="card space-y-3">
    <h2 class="section-title">{m.homeQuickLaunch()}</h2>
    <p class="view-desc">{m.homeQuickLaunchHint()}</p>
    <div class="flex flex-wrap items-end gap-3">
      <label class="flex w-60 flex-col gap-1">
        <span class="view-desc">{m.menuProviders()}</span>
        <Select
          ariaLabel={m.menuProviders()}
          bind:value={selectedProvider}
          options={[
            { value: "", label: m.commonNone(), disabled: true },
            ...$providers.map((p) => ({ value: p.id, label: p.name })),
          ]}
        />
      </label>
      <label class="flex w-60 flex-col gap-1">
        <span class="view-desc">{m.menuAccounts()}</span>
        <Select
          ariaLabel={m.menuAccounts()}
          bind:value={selectedAccount}
          options={[
            { value: "", label: m.commonNone(), disabled: true },
            ...$accounts.map((a) => ({ value: a.id, label: a.name })),
          ]}
        />
      </label>
      <button
        class="btn-primary h-10 px-6 text-base"
        disabled={!selectedProvider || !selectedAccount || launching}
        onclick={() => void launch(selectedProvider, selectedAccount)}
      >
        <Play class="size-4" />
        {m.commonLaunch()}
      </button>
    </div>
  </section>

  <section class="space-y-2">
    <h2 class="section-title">{m.homePersonas()}</h2>
    {#if $accounts.length === 0}
      <p class="view-desc">{m.homeNoPersonas()}</p>
    {:else}
      <div class="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3">
        {#each $accounts as account (account.id)}
          {@const isOpen = $openLabels.includes(`persona-${account.id}`)}
          <div class="flex items-center gap-2 rounded-md border border-line bg-surface px-3 py-2.5">
            <button
              type="button"
              class="flex min-w-0 flex-1 items-center gap-3 rounded-md text-left transition-colors duration-150"
              onclick={() => void openWithLast(account)}
            >
              <span
                class="size-3 shrink-0 rounded-full border border-line"
                style:background={account.color ?? "var(--fg-muted)"}
              ></span>
              <span class="min-w-0">
                <span class="block truncate text-sm font-medium">{account.name}</span>
                <span class="block truncate text-xs text-muted">{account.email}</span>
              </span>
            </button>
            {#if isOpen}
              <span class="badge">
                <span class="size-1.5 rounded-full bg-accent-fg"></span>
                {m.commonOpenBadge()}
              </span>
            {/if}
            <Menu items={personaMenu(account)} label={m.commonActions()} />
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section class="space-y-2">
    <h2 class="section-title">{m.homeRecent()}</h2>
    {#if $recent.length === 0}
      <p class="view-desc">{m.homeRecentEmpty()}</p>
    {:else}
      <ul class="space-y-1.5">
        {#each $recent.slice(0, 8) as entry (`${entry.ts}-${entry.personaId}`)}
          {@const Icon = categoryIcon(entry.providerId)}
          <li class="row py-1.5">
            <Icon class="size-4 shrink-0 text-muted" />
            <span class="min-w-0 flex-1 truncate text-sm">
              {m.homeRecentItem({ persona: entry.personaName, provider: entry.providerName })}
            </span>
            <span class="shrink-0 text-xs text-muted">{formatRelative(entry.ts, $uiLocale)}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

{#if editOpen && editAccount}
  <AccountDialog account={editAccount} onclose={() => (editOpen = false)} />
{/if}

{#if confirmTarget}
  <ConfirmDelete
    name={confirmTarget.name}
    title={confirmMode === "clear" ? m.commonClearData() : m.commonDelete()}
    busy={confirmBusy}
    error={confirmError}
    onconfirm={() => void onConfirm()}
    oncancel={() => (confirmTarget = null)}
  />
{/if}
