<script lang="ts">
  import { Check, Copy } from "@lucide/svelte";
  import { clearAllPersonaData, clearPersonaData, saveProviders, saveSettings } from "../api";
  import { copyText } from "../clipboard";
  import { commandError } from "../errors";
  import { m } from "../paraglide/messages";
  import { setLocale } from "../paraglide/runtime";
  import { activeMenu, applyLocale, applyTheme, accounts, providers, settings, uiLocale } from "../stores";
  import { toast } from "../toast";
  import { REPO_URL, type Settings } from "../types";
  import { get } from "svelte/store";
  import ConfirmDelete from "../ui/ConfirmDelete.svelte";

  const engineOptions = ["webview", "chrome"] as const;
  const engineLabels = $derived({ webview: m.engineWebview(), chrome: m.engineChrome() });

  let confirmTarget = $state<{ id: string; name: string; mode: "clear" | "clearAll" } | null>(
    null,
  );
  let confirmBusy = $state(false);
  let confirmError = $state("");
  let savedFlash = $state(false);

  /** Apply one settings patch immediately: persist, then re-render theme/locale. */
  async function patch(part: Partial<Settings>) {
    const prev = get(settings);
    const next = { ...prev, ...part };
    settings.set(next);
    applyTheme();
    if (part.language) {
      const locale = applyLocale();
      await setLocale(locale);
      uiLocale.set(locale);
    }
    try {
      await saveSettings(next);
      savedFlash = true;
      setTimeout(() => (savedFlash = false), 2000);
    } catch (e) {
      settings.set(prev);
      applyTheme();
      // Revert the live locale the same way the success branch resolved it.
      if (part.language) {
        const locale = applyLocale();
        await setLocale(locale);
        uiLocale.set(locale);
      }
      toast(commandError(e));
    }
  }

  async function setEngine(providerId: string, engine: "webview" | "chrome") {
    const next = $providers.map((p) => (p.id === providerId ? { ...p, engine } : p));
    try {
      await saveProviders(next);
      providers.set(next);
    } catch (e) {
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
    <div class="grid grid-cols-2 gap-x-6 gap-y-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">{m.settingsLanguage()}</span>
        <select
          class="input"
          value={$settings.language}
          onchange={(e) => void patch({ language: e.currentTarget.value as Settings["language"] })}
        >
          <option value="system">{m.settingsSystem()}</option>
          <option value="id">Bahasa Indonesia</option>
          <option value="en">English</option>
        </select>
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">{m.settingsTheme()}</span>
        <select
          class="input"
          value={$settings.theme}
          onchange={(e) => void patch({ theme: e.currentTarget.value as Settings["theme"] })}
        >
          <option value="system">{m.settingsSystem()}</option>
          <option value="light">{m.settingsLight()}</option>
          <option value="dark">{m.settingsDark()}</option>
        </select>
      </label>
    </div>
  </section>

  <section class="card space-y-3">
    <h2 class="section-title">{m.settingsWindowSection()}</h2>
    <div class="grid grid-cols-2 gap-x-6 gap-y-4">
      <label class="flex flex-col gap-1">
        <span class="text-sm font-medium">{m.settingsCloseBehavior()}</span>
        <select
          class="input"
          value={$settings.closeBehavior}
          onchange={(e) =>
            void patch({ closeBehavior: e.currentTarget.value as Settings["closeBehavior"] })}
        >
          <option value="exit">{m.settingsCloseExit()}</option>
          <option value="hide">{m.settingsCloseHide()}</option>
        </select>
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

  <section class="card space-y-2">
    <div class="flex items-center justify-between gap-2">
      <h2 class="section-title">{m.settingsEnginePerProvider()}</h2>
      <button class="btn h-8 text-xs" onclick={() => activeMenu.set("providers")}>
        {m.settingsEditInProviders()}
      </button>
    </div>
    <ul class="space-y-1.5">
      {#each $providers as provider (provider.id)}
        <li class="row py-1.5">
          <span class="min-w-0 flex-1 truncate text-sm">{provider.name}</span>
          <span class="text-xs text-muted">{engineLabels[provider.engine]}</span>
          <select
            class="input w-40"
            value={provider.engine}
            onchange={(e) => void setEngine(provider.id, e.currentTarget.value as "webview" | "chrome")}
          >
            {#each engineOptions as option (option)}
              <option value={option}>{engineLabels[option]}</option>
            {/each}
          </select>
        </li>
      {/each}
    </ul>
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

  <section class="card space-y-2">
    <h2 class="section-title">{m.settingsAbout()}</h2>
    <div class="flex items-center justify-between gap-2">
      <span class="text-sm">{m.settingsVersion()}</span>
      <span class="font-mono text-xs text-muted">{__APP_VERSION__}</span>
    </div>
    <div class="flex items-center justify-between gap-2">
      <span class="text-sm">{m.settingsRepository()}</span>
      <span class="flex items-center gap-1">
        <span class="font-mono text-xs text-muted">{REPO_URL}</span>
        <button
          type="button"
          class="btn-icon"
          aria-label={m.commonCopy()}
          onclick={() => void copyText(REPO_URL).then(() => toast(m.commonCopied()))}
        >
          <Copy class="size-4" />
        </button>
      </span>
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
