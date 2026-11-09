<script lang="ts">
import { Pencil, Plus, Trash2, X } from "@lucide/svelte";
import { saveAccounts } from "../api";
import { m } from "../paraglide/messages";
import { accounts } from "../stores";
import type { Account } from "../types";

let adding = $state(false);
let editingId = $state("");
let draft = $state({ name: "", email: "", color: "" });
let saving = $state(false);
let error = $state("");

function startAdd() {
	editingId = "";
	draft = { name: "", email: "", color: "" };
	adding = true;
	error = "";
}

function startEdit(account: Account) {
	adding = false;
	editingId = account.id;
	draft = { name: account.name, email: account.email, color: account.color ?? "" };
	error = "";
}

function cancel() {
	adding = false;
	editingId = "";
	error = "";
}

function validate(): boolean {
	if (!draft.name.trim() || !draft.email.trim()) {
		error = m.errorRequired();
		return false;
	}
	if (!draft.email.includes("@")) {
		error = m.errorInvalidEmail();
		return false;
	}
	error = "";
	return true;
}

async function persist(next: Account[]) {
	saving = true;
	try {
		await saveAccounts(next);
		accounts.set(next);
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
		const next = $accounts.map((a) =>
			a.id === editingId
				? { ...a, name: draft.name.trim(), email: draft.email.trim(), color: draft.color || null }
				: a,
		);
		void persist(next);
	} else {
		const account: Account = {
			id: crypto.randomUUID(),
			name: draft.name.trim(),
			email: draft.email.trim(),
			color: draft.color || null,
			createdAt: new Date().toISOString(),
		};
		void persist([...$accounts, account]);
	}
}

function remove(account: Account) {
	if (!window.confirm(m.commonConfirmDelete({ name: account.name }))) return;
	void persist($accounts.filter((a) => a.id !== account.id));
}

const showForm = $derived(adding || editingId !== "");
</script>

<div class="view">
  <div class="flex items-center justify-between">
    <div class="space-y-1">
      <h1 class="view-title">{m.accountsTitle()}</h1>
      <p class="view-desc">{m.accountsDescription()}</p>
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
      <div class="grid grid-cols-[1fr_1fr_auto_auto] items-end gap-3">
        <label class="flex flex-col gap-1">
          <span class="view-desc">{m.commonName()}</span>
          <input class="input" bind:value={draft.name} required />
        </label>
        <label class="flex flex-col gap-1">
          <span class="view-desc">{m.commonEmail()}</span>
          <input class="input" type="email" bind:value={draft.email} required />
        </label>
        <label class="flex flex-col gap-1">
          <span class="view-desc">{m.commonColor()}</span>
          <input class="input h-9 w-14 p-1" type="color" bind:value={draft.color} />
        </label>
        <div class="flex gap-2">
          <button class="btn-primary" type="submit" disabled={saving}>{m.commonSave()}</button>
          <button class="btn" type="button" onclick={cancel}>
            <X class="size-4" />
            {m.commonCancel()}
          </button>
        </div>
      </div>
      {#if error}<p class="error-text">{error}</p>{/if}
    </form>
  {/if}

  {#if $accounts.length === 0}
    <p class="view-desc">{m.accountsEmpty()}</p>
  {:else}
    <ul class="space-y-1.5">
      {#each $accounts as account (account.id)}
        <li class="row py-1.5">
          <span
            class="size-3 shrink-0 rounded-full border border-line"
            style:background={account.color ?? "var(--fg-muted)"}
          ></span>
          <span class="min-w-0 flex-1 truncate text-sm">{account.name}</span>
          <span class="min-w-0 flex-1 truncate text-xs text-muted">{account.email}</span>
          <button class="btn-icon" aria-label={m.commonEdit()} onclick={() => startEdit(account)}>
            <Pencil class="size-4" />
          </button>
          <button class="btn-icon hover:text-danger" aria-label={m.commonDelete()} onclick={() => remove(account)}>
            <Trash2 class="size-4" />
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
