<script lang="ts">
  import {
    BookMarked,
    Globe,
    House,
    Monitor,
    Moon,
    Settings as SettingsIcon,
    Sparkles,
    Sun,
    Users,
  } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { listen } from "@tauri-apps/api/event";
  import { commandError } from "./lib/errors";
  import { lastUsedProvider, launchPersona, refreshOpenPersonas } from "./lib/launch";
  import { m } from "./lib/paraglide/messages";
  import { setLocale } from "./lib/paraglide/runtime";
  import { activeMenu, accounts, applyTheme, initSystemListeners, loadAll, providers, recent, settings, uiLocale } from "./lib/stores";
  import { toast } from "./lib/toast";
  import type { Account, MenuId, Settings, ThemeSetting } from "./lib/types";
  import { saveSettings } from "./lib/api";
  import Accounts from "./lib/views/Accounts.svelte";
  import Generate from "./lib/views/Generate.svelte";
  import Home from "./lib/views/Home.svelte";
  import Prompts from "./lib/views/Prompts.svelte";
  import Providers from "./lib/views/Providers.svelte";
  import SettingsView from "./lib/views/Settings.svelte";
  import Toaster from "./lib/ui/Toaster.svelte";

  type MenuEntry = { id: MenuId; icon: typeof House };

  const menus: MenuEntry[] = [
    { id: "home", icon: House },
    { id: "accounts", icon: Users },
    { id: "providers", icon: Globe },
    { id: "generate", icon: Sparkles },
    { id: "prompts", icon: BookMarked },
    { id: "settings", icon: SettingsIcon },
  ];

  const menuLabel: Record<MenuId, () => string> = {
    home: m.menuHome,
    accounts: m.menuAccounts,
    providers: m.menuProviders,
    generate: m.menuGenerate,
    prompts: m.menuPrompts,
    settings: m.menuSettings,
  };

  const themeIcons: Record<ThemeSetting, typeof Sun> = {
    system: Monitor,
    light: Sun,
    dark: Moon,
  };

  const THEME_ORDER: ThemeSetting[] = ["system", "light", "dark"];

  const ThemeIcon = $derived(themeIcons[$settings.theme]);

  onMount(() => {
    const stop = initSystemListeners();
    loadAll()
      .then(() => setLocale(get(uiLocale)))
      .catch((e) => console.error("init failed", e));
    void refreshOpenPersonas();
    // Persona windows can be closed outside this view; Rust emits this event
    // so the Home badge stays accurate.
    let unlisten: (() => void) | undefined;
    void listen("persona-window-closed", () => void refreshOpenPersonas()).then((fn) => {
      unlisten = fn;
    });
    return () => {
      stop();
      unlisten?.();
    };
  });

  /** 3-state cycle: system → light → dark → system. */
  async function cycleTheme() {
    const current = get(settings).theme;
    const next = THEME_ORDER[(THEME_ORDER.indexOf(current) + 1) % THEME_ORDER.length];
    const prev = get(settings);
    const nextSettings: Settings = { ...prev, theme: next };
    settings.set(nextSettings);
    applyTheme();
    try {
      await saveSettings(nextSettings);
    } catch (e) {
      settings.set(prev);
      applyTheme();
      toast(commandError(e));
    }
  }

  /** Ctrl+1..9: open the nth persona (list order) with its last-used provider. */
  async function quickOpen(account: Account) {
    const provider = lastUsedProvider(account, get(recent), get(providers)) ?? get(providers)[0];
    if (!provider) return;
    try {
      await launchPersona(account, provider);
    } catch (e) {
      toast(commandError(e));
    }
  }

  function isTypingTarget(target: EventTarget | null): boolean {
    return (
      target instanceof HTMLElement &&
      (target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.tagName === "SELECT" ||
        target.isContentEditable)
    );
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key >= "1" && e.key <= "9" && !isTypingTarget(e.target)) {
      const account = get(accounts)[Number(e.key) - 1];
      if (account) {
        e.preventDefault();
        void quickOpen(account);
      }
    } else if (e.key === "/" && !e.ctrlKey && !e.altKey && !isTypingTarget(e.target)) {
      // Focus the search box of the current view, if it has one.
      const search = document.querySelector<HTMLInputElement>("input[data-search-input]");
      if (search) {
        e.preventDefault();
        search.focus();
      }
    }
  }
</script>

<svelte:window onkeydown={onkeydown} />

<div class="flex h-screen bg-bg text-fg">
  <nav class="flex w-[220px] shrink-0 flex-col border-r border-line bg-surface">
    <div class="flex h-12 items-center gap-2 border-b border-line px-4">
      <span class="inline-block size-4 rounded-md bg-accent"></span>
      <span class="text-sm font-semibold tracking-wide">BITS Flow AI</span>
    </div>
    <ul class="space-y-0.5 border-b border-line p-2">
      {#key $uiLocale}
        {#each menus as entry (entry.id)}
          {@const Icon = entry.icon}
          <li>
            <button
              type="button"
              class="flex h-9 w-full items-center gap-2.5 rounded-md px-3 text-left transition-colors duration-150 {$activeMenu === entry.id
                ? "border-l-2 border-accent bg-surface font-medium"
                : "border-l-2 border-transparent text-fg hover:bg-bg"}"
              aria-current={$activeMenu === entry.id ? "page" : undefined}
              onclick={() => activeMenu.set(entry.id)}
            >
              <Icon class="size-4 shrink-0" />
              <span class="truncate">{menuLabel[entry.id]()}</span>
            </button>
          </li>
        {/each}
      {/key}
    </ul>

    <div class="mt-auto flex items-center justify-between border-t border-line px-4 py-3">
      <button
        type="button"
        class="btn-icon"
        aria-label={m.settingsTheme()}
        title={m.settingsTheme()}
        onclick={() => void cycleTheme()}
      >
        <ThemeIcon class="size-4" />
      </button>
      <span class="chip uppercase" title={m.settingsLanguage()}>{$uiLocale}</span>
    </div>
  </nav>

  <main class="flex-1 overflow-y-auto">
    {#key $uiLocale}
      {#if $activeMenu === "home"}
        <Home />
      {:else if $activeMenu === "accounts"}
        <Accounts />
      {:else if $activeMenu === "providers"}
        <Providers />
      {:else if $activeMenu === "generate"}
        <Generate />
      {:else if $activeMenu === "prompts"}
        <Prompts />
      {:else}
        <SettingsView />
      {/if}
    {/key}
  </main>
</div>

<Toaster />
