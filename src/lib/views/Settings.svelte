<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { clearAllPersonaData, clearPersonaData, saveSettings } from "../api";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { setLocale } from "../paraglide/runtime";
  import { applyLocale, accounts, settings, uiLocale } from "../stores";
  import { toast } from "../toast";
  import type { Settings } from "../types";
  import { get } from "svelte/store";
  import ConfirmDelete from "../ui/ConfirmDelete.svelte";
  import Select from "../ui/Select.svelte";

  let confirmTarget = $state<{ id: string; name: string; mode: "clear" | "clearAll" } | null>(
    null,
  );
  let confirmBusy = $state(false);
  let confirmError = $state("");
  let savedFlash = $state(false);

  /** Apply one settings patch immediately and persist it. */
  async function patch(part: Partial<Settings>) {
    const prev = get(settings);
    const next = { ...prev, ...part };
    settings.set(next);
    if (part.language) {
      const locale = applyLocale();
      await setLocale(locale, { reload: false });
      uiLocale.set(locale);
    }
    try {
      await saveSettings(next);
      savedFlash = true;
      setTimeout(() => (savedFlash = false), 2000);
    } catch (e) {
      settings.set(prev);
      // Revert the live locale the same way the success branch resolved it.
      if (part.language) {
        const locale = applyLocale();
        await setLocale(locale, { reload: false });
        uiLocale.set(locale);
      }
      toast(commandError(e));
    }
  }

  function clampLru(value: number) {
    const clamped = Math.min(10, Math.max(1, Math.round(Number.isNaN(value) ? 3 : value)));
    void patch({ lruSize: clamped });
  }

  async function onConfirm() {
    const target = confirmTarget;
    if (!target || confirmBusy) return;
    confirmBusy = true;
    confirmError = "";
    try {
      if (target.mode === "clearAll") {
        const result = await clearAllPersonaData();
        toast(
          result.failed.length > 0
            ? m.settingsClearFailed({ count: result.failed.length })
            : result.skipped.length > 0
              ? m.settingsSkippedOpen({ count: result.skipped.length })
              : m.settingsAllCleared(),
        );
      } else {
        await clearPersonaData(target.id);
        toast(m.commonDataCleared());
      }
      confirmTarget = null;
    } catch (e) {
      confirmError = commandError(e);
    } finally {
      confirmBusy = false;
    }
  }
</script>

<div class="view">
  <div class="flex items-center justify-between gap-3">
    <h1 class="view-title">{m.settingsTitle()}</h1>
    {#if savedFlash}
      <span class="inline-flex items-center gap-1 text-xs text-accent">
        <Check class="size-4" />
        {m.settingsSaved()}
      </span>
    {/if}
  </div>

  <section class="card space-y-3">
    <h2 class="section-title">{m.settingsGeneral()}</h2>
    <div class="grid grid-cols-3 gap-x-6 gap-y-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">{m.settingsLanguage()}</span>
        <Select
          ariaLabel={m.settingsLanguage()}
          value={$settings.language}
          options={[
            { value: "system", label: m.settingsSystem() },
            { value: "id", label: "Bahasa Indonesia" },
            { value: "en", label: "English" },
          ]}
          onchange={(v) => void patch({ language: v as Settings["language"] })}
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">{m.settingsCloseBehavior()}</span>
        <Select
          ariaLabel={m.settingsCloseBehavior()}
          value={$settings.closeBehavior}
          options={[
            { value: "exit", label: m.settingsCloseExit() },
            { value: "hide", label: m.settingsCloseHide() },
          ]}
          onchange={(v) => void patch({ closeBehavior: v as Settings["closeBehavior"] })}
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">{m.settingsLruSize()}</span>
        <input
          class="input"
          type="number"
          min="1"
          max="10"
          value={$settings.lruSize}
          onchange={(e) => clampLru(e.currentTarget.valueAsNumber)}
        />
      </label>
    </div>
  </section>

  <section class="card space-y-3 border-danger">
    <h2 class="section-title text-danger">{m.settingsDataZone()}</h2>
    <p class="view-desc">{m.settingsDataZoneDesc()}</p>
    <ul class="space-y-1.5">
      {#each $accounts as account (account.id)}
        <li class="row py-1.5">
          <span
            class="size-3 shrink-0 rounded-full border border-line"
            style:background={account.color ?? "var(--fg-muted)"}
          ></span>
          <span class="min-w-0 flex-1 truncate text-sm">{account.name}</span>
          <span class="min-w-0 flex-1 truncate text-xs text-muted">{account.email}</span>
          <button
            class="btn h-8 shrink-0 text-xs"
            onclick={() => {
              confirmTarget = { id: account.id, name: account.name, mode: "clear" };
              confirmError = "";
            }}
          >
            {m.commonClearData()}
          </button>
        </li>
      {/each}
    </ul>
    <div class="flex">
      <button
        class="btn-danger"
        onclick={() => {
          confirmTarget = { id: "", name: m.appName(), mode: "clearAll" };
          confirmError = "";
        }}
      >
        {m.settingsClearAllData()}
      </button>
    </div>
  </section>
</div>

{#if confirmTarget}
  <ConfirmDelete
    name={confirmTarget.name}
    title={confirmTarget.mode === "clearAll" ? m.settingsClearAllData() : m.commonClearData()}
    busy={confirmBusy}
    error={confirmError}
    onconfirm={() => void onConfirm()}
    oncancel={() => (confirmTarget = null)}
  />
{/if}
