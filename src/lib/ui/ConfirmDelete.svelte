<script lang="ts">
  import Modal from "./Modal.svelte";
  import { m } from "../paraglide/messages";

  let {
    name,
    title,
    busy = false,
    error = "",
    onconfirm,
    oncancel,
  }: {
    name: string;
    title: string;
    busy?: boolean;
    error?: string;
    onconfirm: () => void;
    oncancel: () => void;
  } = $props();

  let typed = $state("");
  const ok = $derived(typed === name);
</script>

<Modal {title} onclose={oncancel}>
  <form
    class="space-y-3"
    onsubmit={(e) => {
      e.preventDefault();
      if (ok && !busy) onconfirm();
    }}
  >
    <p class="text-sm">{m.commonConfirmDelete({ name })}</p>
    <p class="view-desc">{m.commonConfirmHint({ name })}</p>
    <input
      class="input font-mono"
      bind:value={typed}
      autocomplete="off"
      spellcheck="false"
      aria-label={name}
    />
    {#if error}<p class="error-text">{error}</p>{/if}
    <div class="flex gap-2">
      <!-- Danger button left, per spec. -->
      <button class="btn-danger" type="submit" disabled={!ok || busy}>{m.commonDelete()}</button>
      <button class="btn" type="button" onclick={oncancel}>{m.commonCancel()}</button>
    </div>
  </form>
</Modal>
