<script lang="ts">
  import { Copy, Download, Pin, PinOff, Plus, Upload } from "@lucide/svelte";
  import { exportFile, importFile, promptSchema, savePrompts } from "../api";
  import { copyText } from "../clipboard";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { categoryLabel } from "../settings";
  import { prompts } from "../stores";
  import { toast } from "../toast";
  import type { Prompt } from "../types";
  import { get } from "svelte/store";
  import ConfirmDelete from "../ui/ConfirmDelete.svelte";
  import { mergeById, parseItems } from "../transfer";

  /** Write every prompt to a user-picked JSON file. */
  async function exportPrompts() {
    try {
      const contents = JSON.stringify({ kind: "prompts", version: 1, items: get(prompts) }, null, 2);
      const stamp = new Date().toISOString().slice(0, 10);
      const saved = await exportFile(`bits-flow-ai-prompts-${stamp}.json`, contents);
      if (saved) toast(m.exportDone());
    } catch (e) {
      toast(commandError(e));
    }
  }

  /** Merge a user-picked prompts export into the current list by id. */
  async function importPrompts() {
    try {
      const raw = await importFile();
      if (raw === null) return;
      const items = parseItems("prompts", promptSchema, raw);
      if (!items) {
        toast(m.errorInvalidFile());
        return;
      }
      const prev = get(prompts);
      const { merged, skipped } = mergeById(prev, items);
      await savePrompts(merged);
      prompts.set(merged);
      toast(m.importSummary({ imported: merged.length - prev.length, skipped }));
    } catch (e) {
      toast(commandError(e));
    }
  }
  import Menu from "../ui/Menu.svelte";
  import PromptDialog from "../ui/PromptDialog.svelte";

  let query = $state("");
  let dialogPrompt = $state<Prompt | null>(null);
  let dialogOpen = $state(false);
  let confirmTarget = $state<Prompt | null>(null);
  let confirmBusy = $state(false);
  let confirmError = $state("");

  const catLabels = $derived({
    image: m.categoryImage(),
    video: m.categoryVideo(),
    both: m.categoryBoth(),
  });

  const sorted = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const base = q
      ? $prompts.filter(
          (p) => p.title.toLowerCase().includes(q) || p.body.toLowerCase().includes(q),
        )
      : $prompts;
    // Pinned first, insertion order preserved within each group.
    return [...base].sort((a, b) => Number(b.pinned) - Number(a.pinned));
  });

  async function togglePin(prompt: Prompt) {
    const next = $prompts.map((p) => (p.id === prompt.id ? { ...p, pinned: !p.pinned } : p));
    try {
      await savePrompts(next);
      prompts.set(next);
    } catch (e) {
      toast(commandError(e));
    }
  }

  async function copy(prompt: Prompt) {
    await copyText(prompt.body);
    toast(m.promptsCopied());
  }

  async function remove(prompt: Prompt) {
    if (confirmBusy) return;
    confirmBusy = true;
    confirmError = "";
    const idx = $prompts.findIndex((p) => p.id === prompt.id);
    const next = $prompts.filter((p) => p.id !== prompt.id);
    try {
      await savePrompts(next);
      prompts.set(next);
      confirmTarget = null;
      toast(m.commonDeletedToast({ name: prompt.title }), {
        label: m.commonUndo(),
        run: () => void restore(prompt, idx),
      });
    } catch (e) {
      confirmError = commandError(e);
    } finally {
      confirmBusy = false;
    }
  }

  async function restore(prompt: Prompt, idx: number) {
    const list = [...get(prompts)];
    list.splice(Math.min(idx, list.length), 0, prompt);
    try {
      await savePrompts(list);
      prompts.set(list);
    } catch (e) {
      toast(commandError(e));
    }
  }
</script>

<div class="view">
  <div class="flex items-center justify-between gap-3">
    <div class="space-y-1">
      <h1 class="view-title">{m.promptsTitle()}</h1>
      <p class="view-desc">{m.promptsDescription()}</p>
    </div>
    <div class="flex items-center gap-2">
      <input
        class="input w-56"
        data-search-input
        placeholder={m.commonSearch()}
        bind:value={query}
      />
      <button
        type="button"
        class="btn-icon"
        aria-label={m.commonExport()}
        title={m.commonExport()}
        onclick={() => void exportPrompts()}
      >
        <Download class="size-4" />
      </button>
      <button
        type="button"
        class="btn-icon"
        aria-label={m.commonImport()}
        title={m.commonImport()}
        onclick={() => void importPrompts()}
      >
        <Upload class="size-4" />
      </button>
      <button
        class="btn-primary"
        onclick={() => {
          dialogPrompt = null;
          dialogOpen = true;
        }}
      >
        <Plus class="size-4" />
        {m.commonAdd()}
      </button>
    </div>
  </div>

  {#if $prompts.length === 0}
    <p class="view-desc">{m.promptsEmpty()}</p>
  {:else if sorted.length === 0}
    <p class="view-desc">{m.commonNoResults()}</p>
  {:else}
    <ul class="space-y-1.5">
      {#each sorted as prompt (prompt.id)}
        <li class="group row items-start py-2 pr-1.5">
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              {#if prompt.pinned}<Pin class="size-3.5 shrink-0 text-accent" />{/if}
              <span class="truncate text-sm font-medium">{prompt.title}</span>
              <span class="chip shrink-0">{categoryLabel(prompt.category, catLabels)}</span>
            </div>
            <p class="mt-0.5 line-clamp-2 text-xs whitespace-pre-line text-muted">{prompt.body}</p>
          </div>
          <!-- Copy on hover; stays reachable via keyboard focus. -->
          <button
            type="button"
            class="btn-icon opacity-0 transition-opacity duration-150 group-hover:opacity-100 focus-visible:opacity-100"
            aria-label={m.commonCopy()}
            onclick={() => void copy(prompt)}
          >
            <Copy class="size-4" />
          </button>
          <button
            type="button"
            class="btn-icon"
            aria-label={m.promptsPinned()}
            onclick={() => void togglePin(prompt)}
          >
            {#if prompt.pinned}<PinOff class="size-4" />{:else}<Pin class="size-4" />{/if}
          </button>
          <Menu
            label={m.commonActions()}
            items={[
              {
                kind: "action" as const,
                label: m.commonEdit(),
                onselect: () => {
                  dialogPrompt = prompt;
                  dialogOpen = true;
                },
              },
              {
                kind: "action" as const,
                label: m.commonCopy(),
                onselect: () => void copy(prompt),
              },
              {
                kind: "action" as const,
                label: m.commonDelete(),
                danger: true,
                onselect: () => {
                  confirmTarget = prompt;
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
  <PromptDialog prompt={dialogPrompt} onclose={() => (dialogOpen = false)} />
{/if}

{#if confirmTarget}
  <ConfirmDelete
    name={confirmTarget.title}
    title={m.commonDelete()}
    busy={confirmBusy}
    error={confirmError}
    onconfirm={() => void remove(confirmTarget!)}
    oncancel={() => (confirmTarget = null)}
  />
{/if}
