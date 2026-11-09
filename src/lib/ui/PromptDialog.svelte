<script lang="ts">
  import { savePrompts } from "../api";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { prompts } from "../stores";
  import type { Category, Prompt } from "../types";
  import Modal from "../ui/Modal.svelte";

  let { prompt, onclose }: { prompt: Prompt | null; onclose: () => void } = $props();

  // svelte-ignore state_referenced_locally
  let title = $state(prompt?.title ?? "");
  // svelte-ignore state_referenced_locally
  let body = $state(prompt?.body ?? "");
  // svelte-ignore state_referenced_locally
  let category = $state<Category>(prompt?.category ?? "both");
  // svelte-ignore state_referenced_locally
  let pinned = $state(prompt?.pinned ?? false);
  let saving = $state(false);
  let error = $state("");

  const catLabels = $derived({
    image: m.categoryImage(),
    video: m.categoryVideo(),
    both: m.categoryBoth(),
  });

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    if (saving) return;
    const trimmedTitle = title.trim();
    const trimmedBody = body.trim();
    if (!trimmedTitle || !trimmedBody) {
      error = m.errorRequired();
      return;
    }
    error = "";
    saving = true;
    const next = prompt
      ? $prompts.map((p) =>
          p.id === prompt.id
            ? { ...p, title: trimmedTitle, body: trimmedBody, category, pinned }
            : p,
        )
      : [
          ...$prompts,
          {
            id: crypto.randomUUID(),
            title: trimmedTitle,
            body: trimmedBody,
            category,
            pinned,
          } satisfies Prompt,
        ];
    try {
      await savePrompts(next);
      prompts.set(next);
      onclose();
    } catch (err) {
      error = commandError(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={prompt ? m.commonEdit() : m.commonAdd()} {onclose}>
  <form class="space-y-3" onsubmit={submit}>
    <div class="grid grid-cols-[1fr_auto_auto] items-end gap-3">
      <label class="flex flex-col gap-1">
        <span class="view-desc">{m.commonTitle()}</span>
        <input class="input" bind:value={title} required />
      </label>
      <label class="flex w-36 flex-col gap-1">
        <span class="view-desc">{m.commonCategory()}</span>
        <select class="input" bind:value={category}>
          {#each ["image", "video", "both"] as c (c)}
            <option value={c}>{catLabels[c as Category]}</option>
          {/each}
        </select>
      </label>
      <label class="flex h-9 items-center gap-2 pb-0.5">
        <input type="checkbox" class="size-4 accent-(--accent)" bind:checked={pinned} />
        <span class="text-sm">{m.promptsPinned()}</span>
      </label>
    </div>
    <label class="flex flex-col gap-1">
      <span class="view-desc">{m.commonBody()}</span>
      <textarea class="input min-h-32 py-2 font-mono" bind:value={body} required></textarea>
    </label>
    {#if error}<p class="error-text">{error}</p>{/if}
    <div class="flex gap-2">
      <button class="btn-primary" type="submit" disabled={saving}>{m.commonSave()}</button>
      <button class="btn" type="button" onclick={onclose}>{m.commonCancel()}</button>
    </div>
  </form>
</Modal>
