<script lang="ts">
  import { z } from "zod";
  import { saveAccounts } from "../api";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { accounts } from "../stores";
  import { PERSONA_COLORS, type Account } from "../types";
  import Modal from "../ui/Modal.svelte";

  let { account, onclose }: { account: Account | null; onclose: () => void } = $props();

  // svelte-ignore state_referenced_locally
  let name = $state(account?.name ?? "");
  // svelte-ignore state_referenced_locally
  let email = $state(account?.email ?? "");
  // svelte-ignore state_referenced_locally
  let color = $state(account?.color ?? PERSONA_COLORS[0]);
  let saving = $state(false);
  let error = $state("");

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    if (saving) return;
    const trimmedName = name.trim();
    if (!trimmedName) {
      error = m.errorRequired();
      return;
    }
    if (!z.email().safeParse(email.trim()).success) {
      error = m.errorInvalidEmail();
      return;
    }
    error = "";
    saving = true;
    const next = account
      ? $accounts.map((a) =>
          a.id === account.id
            ? { ...a, name: trimmedName, email: email.trim(), color }
            : a,
        )
      : [
          ...$accounts,
          {
            id: crypto.randomUUID(),
            name: trimmedName,
            email: email.trim(),
            color,
            createdAt: new Date().toISOString(),
          } satisfies Account,
        ];
    try {
      await saveAccounts(next);
      accounts.set(next);
      onclose();
    } catch (err) {
      error = commandError(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={account ? m.commonEdit() : m.commonAdd()} {onclose}>
  <form class="space-y-3" onsubmit={submit}>
    <label class="flex flex-col gap-1">
      <span class="view-desc">{m.commonName()}</span>
      <input class="input" bind:value={name} required />
    </label>
    <label class="flex flex-col gap-1">
      <span class="view-desc">{m.commonEmail()}</span>
      <input class="input" type="email" bind:value={email} required />
    </label>
    <div class="flex flex-col gap-1">
      <span class="view-desc">{m.commonColor()}</span>
      <div class="flex gap-1.5">
        {#each PERSONA_COLORS as c (c)}
          <button
            type="button"
            class="size-6 rounded-full border transition-opacity duration-150 hover:opacity-80 {color === c
              ? "ring-2 ring-(--accent) ring-offset-2 ring-offset-(--surface)"
              : "border-line"}"
            style:background={c}
            aria-label={c}
            aria-pressed={color === c}
            onclick={() => (color = c)}
          ></button>
        {/each}
      </div>
    </div>
    {#if error}<p class="error-text">{error}</p>{/if}
    <div class="flex gap-2">
      <button class="btn-primary" type="submit" disabled={saving}>{m.commonSave()}</button>
      <button class="btn" type="button" onclick={onclose}>{m.commonCancel()}</button>
    </div>
  </form>
</Modal>
