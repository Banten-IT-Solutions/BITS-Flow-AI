<script lang="ts">
  import { savePrompts } from "../api";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { prompts } from "../stores";
  import type { Category, Prompt } from "../types";
  import Modal from "../ui/Modal.svelte";
  import Select from "../ui/Select.svelte";

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
  <form class="space-y-4" onsubmit={submit}>
    <label class="flex flex-col gap-1.5">
      <span class="text-sm font-medium">{m.commonTitle()}</span>
      <input class="input" bind:value={title} required />
    </label>
    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1.5">
        <span class="text-sm font-medium">{m.commonCategory()}</span>
        <Select
          ariaLabel={m.commonCategory()}
          bind:value={category}
          options={(["image", "video", "both"] as Category[]).map((c) => ({
            value: c,
            label: catLabels[c],
          }))}
        />
      </label>
      <div class="flex flex-col gap-1.5">
        <span class="text-sm font-medium">{m.promptsPinned()}</span>
        <label class="flex h-9 cursor-pointer items-center select-none">
          <input type="checkbox" class="peer sr-only" bind:checked={pinned} />
          <!-- Toggle switch: track + sliding knob, accent when pinned. -->
          <span
            class="relative inline-flex h-5 w-9 shrink-0 rounded-full border border-line bg-line transition-colors duration-150 after:absolute after:top-0.5 after:left-0.5 after:size-3.5 after:rounded-full after:bg-fg after:transition-transform after:duration-150 peer-checked:border-accent peer-checked:bg-accent peer-checked:after:translate-x-4 peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-accent"
          ></span>
        </label>
      </div>
    </div>
    <label class="flex flex-col gap-1.5">
      <span class="text-sm font-medium">{m.commonBody()}</span>
      <textarea class="input min-h-36 resize-y py-2 font-mono" bind:value={body} required
      ></textarea>
    </label>
    {#if error}<p class="error-text">{error}</p>{/if}
    <div class="flex justify-end gap-2 pt-1">
      <button class="btn" type="button" onclick={onclose}>{m.commonCancel()}</button>
      <button class="btn-primary" type="submit" disabled={saving}>{m.commonSave()}</button>
    </div>
  </form>
</Modal>
