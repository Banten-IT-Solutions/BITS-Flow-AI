<script lang="ts">
  import { Plus } from "@lucide/svelte";
  import { saveAccounts } from "../api";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { accounts, recent, uiLocale } from "../stores";
  import { toast } from "../toast";
  import { formatRelative } from "../time";
  import type { Account } from "../types";
  import { get } from "svelte/store";
  import AccountDialog from "../ui/AccountDialog.svelte";
  import ConfirmDelete from "../ui/ConfirmDelete.svelte";
  import EmptyState from "../ui/EmptyState.svelte";
  import Menu from "../ui/Menu.svelte";
  import { Search, Users } from "@lucide/svelte";

  let query = $state("");
  let dialogAccount = $state<Account | null>(null);
  let dialogOpen = $state(false);
  let confirmTarget = $state<Account | null>(null);
  let confirmBusy = $state(false);
  let confirmError = $state("");

  const filtered = $derived(
    query.trim()
      ? $accounts.filter(
          (a) =>
            a.name.toLowerCase().includes(query.trim().toLowerCase()) ||
            a.email.toLowerCase().includes(query.trim().toLowerCase()),
        )
      : $accounts,
  );

  function lastUsedTs(account: Account): number | null {
    const latest = $recent.find((r) => r.personaId === account.id);
    return latest ? latest.ts : null;
  }

  async function remove(account: Account) {
    if (confirmBusy) return;
    confirmBusy = true;
    confirmError = "";
    const idx = $accounts.findIndex((a) => a.id === account.id);
    const next = $accounts.filter((a) => a.id !== account.id);
    try {
      await saveAccounts(next);
      accounts.set(next);
      confirmTarget = null;
      toast(m.commonDeletedToast({ name: account.name }), {
        label: m.commonUndo(),
        run: () => void restore(account, idx),
      });
    } catch (e) {
      confirmError = commandError(e);
    } finally {
      confirmBusy = false;
    }
  }

  async function restore(account: Account, idx: number) {
    const list = [...get(accounts)];
    list.splice(Math.min(idx, list.length), 0, account);
    try {
      await saveAccounts(list);
      accounts.set(list);
    } catch (e) {
      toast(commandError(e));
    }
  }
</script>

<div class="view">
  <div class="flex items-center justify-between gap-3">
    <div class="space-y-1">
      <h1 class="view-title">{m.accountsTitle()}</h1>
      <p class="view-desc">{m.accountsDescription()}</p>
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
          dialogAccount = null;
          dialogOpen = true;
        }}
      >
        <Plus class="size-4" />
        {m.commonAdd()}
      </button>
    </div>
  </div>

  {#if $accounts.length === 0}
    <EmptyState icon={Users} message={m.accountsEmpty()}>
      <button
        class="btn-primary"
        onclick={() => {
          dialogAccount = null;
          dialogOpen = true;
        }}
      >
        <Plus class="size-4" />
        {m.commonAdd()}
      </button>
    </EmptyState>
  {:else if filtered.length === 0}
    <EmptyState icon={Search} message={m.commonNoResults()}>
      <button class="btn" onclick={() => (query = "")}>{m.commonCancel()}</button>
    </EmptyState>
  {:else}
    <ul class="space-y-1.5">
      {#each filtered as account (account.id)}
        {@const ts = lastUsedTs(account)}
        <li class="row pr-1.5">
          <span
            class="size-3 shrink-0 rounded-full border border-line"
            style:background={account.color ?? "var(--fg-muted)"}
          ></span>
          <span class="min-w-0 flex-[2] truncate text-sm">{account.name}</span>
          <span class="min-w-0 flex-[2] truncate text-xs text-muted">{account.email}</span>
          <span class="w-36 shrink-0 truncate text-right text-xs text-muted">
            {ts === null
              ? m.commonNeverUsed()
              : m.commonLastUsed({ time: formatRelative(ts, $uiLocale) })}
          </span>
          <Menu
            label={m.commonActions()}
            items={[
              {
                kind: "action" as const,
                label: m.commonEdit(),
                onselect: () => {
                  dialogAccount = account;
                  dialogOpen = true;
                },
              },
              {
                kind: "action" as const,
                label: m.commonDelete(),
                danger: true,
                onselect: () => {
                  confirmTarget = account;
                  confirmError = "";
                },
              },
            ]}
          />
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if dialogOpen}
  <AccountDialog account={dialogAccount} onclose={() => (dialogOpen = false)} />
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
