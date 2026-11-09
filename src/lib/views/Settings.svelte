<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { saveProviders, saveSettings } from "../api";
  import { m } from "../paraglide/messages";
  import { setLocale } from "../paraglide/runtime";
  import { applyLocale, applyTheme, providers, settings, uiLocale } from "../stores";

  let language = $state<"system" | "id" | "en">("system");
  let theme = $state<"system" | "light" | "dark">("system");
  let closeBehavior = $state<"exit" | "hide">("exit");
  let lruSize = $state(3);
  let saving = $state(false);
  let error = $state("");
  let saved = $state(false);

  const engineOptions = ["webview", "chrome"] as const;
  const engineLabels = $derived({ webview: m.engineWebview(), chrome: m.engineChrome() });

  // Sync draft when the store loads or changes elsewhere.
  $effect(() => {
    const s = $settings;
    language = s.language;
    theme = s.theme;
    closeBehavior = s.closeBehavior;
    lruSize = s.lruSize;
  });

  async function save() {
    if (saving) return;
    saving = true;
    error = "";
    try {
      const next = { language, theme, closeBehavior, lruSize };
      await saveSettings(next);
      settings.set(next);
      applyTheme();
      const locale = applyLocale();
      await setLocale(locale);
      uiLocale.set(locale);
      saved = true;
      setTimeout(() => (saved = false), 2000);
    } catch (e) {
      error = m.errorCommandFailed({ message: String(e) });
    } finally {
      saving = false;
    }
  }

  async function setEngine(providerId: string, engine: "webview" | "chrome") {
    const next = $providers.map((p) => (p.id === providerId ? { ...p, engine } : p));
    try {
      await saveProviders(next);
      providers.set(next);
    } catch (e) {
      error = m.errorCommandFailed({ message: String(e) });
    }
  }
</script>

<div class="view">
  <h1 class="view-title">{m.settingsTitle()}</h1>

  <section class="card grid grid-cols-2 gap-x-6 gap-y-4">
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">{m.settingsLanguage()}</span>
      <select class="input" bind:value={language}>
        <option value="system">{m.settingsSystem()}</option>
        <option value="id">Bahasa Indonesia</option>
        <option value="en">English</option>
      </select>
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">{m.settingsTheme()}</span>
      <select class="input" bind:value={theme}>
        <option value="system">{m.settingsSystem()}</option>
        <option value="light">{m.settingsLight()}</option>
        <option value="dark">{m.settingsDark()}</option>
      </select>
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">{m.settingsCloseBehavior()}</span>
      <select class="input" bind:value={closeBehavior}>
        <option value="exit">{m.settingsCloseExit()}</option>
        <option value="hide">{m.settingsCloseHide()}</option>
      </select>
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium">{m.settingsLruSize()}</span>
      <input class="input" type="number" min="1" max="10" bind:value={lruSize} />
    </label>
    <div class="col-span-2 flex items-center gap-3">
      <button class="btn-primary" onclick={save} disabled={saving}>{m.commonSave()}</button>
      {#if saved}
        <span class="inline-flex items-center gap-1 text-xs text-accent">
          <Check class="size-4" />
          {m.settingsSaved()}
        </span>
      {/if}
      {#if error}<span class="error-text">{error}</span>{/if}
    </div>
  </section>

  <section class="card space-y-2">
    <h2 class="text-sm font-medium">{m.settingsEnginePerProvider()}</h2>
    <ul class="space-y-1.5">
      {#each $providers as provider (provider.id)}
        <li class="row py-1.5">
          <span class="min-w-0 flex-1 truncate text-sm">{provider.name}</span>
          <span class="text-xs text-muted">{engineLabels[provider.engine]}</span>
          <select
            class="input w-40"
            value={provider.engine}
            onchange={(e) => setEngine(provider.id, e.currentTarget.value as "webview" | "chrome")}
          >
            {#each engineOptions as option (option)}
              <option value={option}>{engineLabels[option]}</option>
            {/each}
          </select>
        </li>
      {/each}
    </ul>
  </section>
</div>
