<script lang="ts">
import { Pencil, Pin, PinOff, Plus, Trash2, X } from "@lucide/svelte";
import { savePrompts } from "../api";
import { m } from "../paraglide/messages";
import { categoryLabel } from "../settings";
import { prompts } from "../stores";
import type { Category, Prompt } from "../types";

let adding = $state(false);
let editingId = $state("");
let draft = $state({ title: "", body: "", category: "both" as Category, pinned: false });
let saving = $state(false);
let error = $state("");

const catLabels = $derived({
	image: m.categoryImage(),
	video: m.categoryVideo(),
	both: m.categoryBoth(),
});

const sorted = $derived([...$prompts].sort((a, b) => Number(b.pinned) - Number(a.pinned)));

function startAdd() {
	editingId = "";
	draft = { title: "", body: "", category: "both", pinned: false };
	adding = true;
	error = "";
}

function startEdit(prompt: Prompt) {
	adding = false;
	editingId = prompt.id;
	draft = {
		title: prompt.title,
		body: prompt.body,
		category: prompt.category,
		pinned: prompt.pinned,
	};
	error = "";
}

function cancel() {
	adding = false;
	editingId = "";
	error = "";
}

async function persist(next: Prompt[]) {
	saving = true;
	try {
		await savePrompts(next);
		prompts.set(next);
		cancel();
	} catch (e) {
		error = m.errorCommandFailed({ message: String(e) });
	} finally {
		saving = false;
	}
}

function submit() {
	if (!draft.title.trim() || !draft.body.trim() || saving) {
		if (!draft.title.trim() || !draft.body.trim()) error = m.errorRequired();
		return;
	}
	error = "";
	if (editingId) {
		const next = $prompts.map((p) =>
			p.id === editingId
				? {
						...p,
						title: draft.title.trim(),
						body: draft.body.trim(),
						category: draft.category,
						pinned: draft.pinned,
					}
				: p,
		);
		void persist(next);
	} else {
		const prompt: Prompt = {
			id: crypto.randomUUID(),
			title: draft.title.trim(),
			body: draft.body.trim(),
			category: draft.category,
			pinned: draft.pinned,
		};
		void persist([...$prompts, prompt]);
	}
}

function remove(prompt: Prompt) {
	if (!window.confirm(m.commonConfirmDelete({ name: prompt.title }))) return;
	void persist($prompts.filter((p) => p.id !== prompt.id));
}

const showForm = $derived(adding || editingId !== "");
</script>

<div class="view">
  <div class="flex items-center justify-between">
    <div class="space-y-1">
      <h1 class="view-title">{m.promptsTitle()}</h1>
      <p class="view-desc">{m.promptsDescription()}</p>
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
      <div class="flex items-end gap-3">
        <label class="flex flex-1 flex-col gap-1">
          <span class="view-desc">{m.commonTitle()}</span>
          <input class="input" bind:value={draft.title} required />
        </label>
        <label class="flex w-40 flex-col gap-1">
          <span class="view-desc">{m.commonCategory()}</span>
          <select class="input" bind:value={draft.category}>
            <option value="image">{catLabels.image}</option>
            <option value="video">{catLabels.video}</option>
            <option value="both">{catLabels.both}</option>
          </select>
        </label>
        <label class="flex h-9 items-center gap-2">
          <input type="checkbox" class="size-4 accent-(--accent)" bind:checked={draft.pinned} />
          <span class="text-sm">{m.promptsPinned()}</span>
        </label>
      </div>
      <label class="flex flex-col gap-1">
        <span class="view-desc">{m.commonBody()}</span>
        <textarea class="input min-h-24 py-2" bind:value={draft.body} required></textarea>
      </label>
      <div class="flex gap-2">
        <button class="btn-primary" type="submit" disabled={saving}>{m.commonSave()}</button>
        <button class="btn" type="button" onclick={cancel}>
          <X class="size-4" />
          {m.commonCancel()}
        </button>
      </div>
      {#if error}<p class="error-text">{error}</p>{/if}
    </form>
  {/if}

  {#if $prompts.length === 0}
    <p class="view-desc">{m.promptsEmpty()}</p>
  {:else}
    <ul class="space-y-1.5">
      {#each sorted as prompt (prompt.id)}
        <li class="row items-start py-2">
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              {#if prompt.pinned}<Pin class="size-3.5 text-accent" />{/if}
              <span class="truncate text-sm font-medium">{prompt.title}</span>
              <span class="text-xs text-muted">{categoryLabel(prompt.category, catLabels)}</span>
            </div>
            <p class="mt-0.5 line-clamp-2 text-xs text-muted whitespace-pre-line">{prompt.body}</p>
          </div>
          <button
            class="btn-icon"
            aria-label={prompt.pinned ? m.promptsPinned() : m.promptsPinned()}
            onclick={() => void persist($prompts.map((p) => (p.id === prompt.id ? { ...p, pinned: !p.pinned } : p)))}
          >
            {#if prompt.pinned}<PinOff class="size-4" />{:else}<Pin class="size-4" />{/if}
          </button>
          <button class="btn-icon" aria-label={m.commonEdit()} onclick={() => startEdit(prompt)}>
            <Pencil class="size-4" />
          </button>
          <button class="btn-icon hover:text-danger" aria-label={m.commonDelete()} onclick={() => remove(prompt)}>
            <Trash2 class="size-4" />
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
