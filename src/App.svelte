<script lang="ts">
  import {
    ArrowUpCircle,
    BookMarked,
    Globe,
    House,
    Settings as SettingsIcon,
    Sparkles,
    Users,
  } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { listen } from "@tauri-apps/api/event";
  import { commandError } from "./lib/errors";
  import { lastUsedProvider, launchPersona, refreshOpenPersonas } from "./lib/launch";
  import { openReleasePage } from "./lib/api";
  import { m } from "./lib/paraglide/messages";
  import { setLocale } from "./lib/paraglide/runtime";
  import { toast } from "./lib/toast";
  import { checkForUpdate } from "./lib/updates";
  import { activeMenu, accounts, loadAll, providers, recent, uiLocale } from "./lib/stores";
  import type { Account, MenuId } from "./lib/types";
  import logo from "./assets/logo.png";
  import pkg from "../package.json";
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

  /** Tag of the newest GitHub release when it is newer than this build, else null. */
  let updateVersion = $state<string | null>(null);

  onMount(() => {
    loadAll()
      .then(() => setLocale(get(uiLocale), { reload: false }))
      .catch((e) => console.error("init failed", e));
    void checkForUpdate(pkg.version).then((tag) => (updateVersion = tag));
    void refreshOpenPersonas();
    // Persona windows can be closed outside this view; Rust emits this event
    // so the Home badge stays accurate.
    let unlisten: (() => void) | undefined;
    void listen("persona-window-closed", () => void refreshOpenPersonas()).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  });

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
      <img src={logo} alt="" class="size-6" />
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

    <div class="mt-auto flex items-center justify-between gap-2 border-t border-line px-4 py-3">
      <div class="min-w-0 leading-tight">
        <div class="truncate text-xs font-semibold">BITS Flow AI</div>
        <div class="truncate text-xs text-muted">{m.appVersion({ version: pkg.version })}</div>
      </div>
      {#if updateVersion}
        <button
          type="button"
          class="badge shrink-0 cursor-pointer transition-opacity duration-150 hover:opacity-90"
          title={m.updateAvailable({ version: updateVersion })}
          onclick={() =>
            openReleasePage().catch((e) => {
              toast(commandError(e));
            })}
        >
          <ArrowUpCircle class="size-3" />
          {m.updateAction()}
        </button>
      {/if}
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
