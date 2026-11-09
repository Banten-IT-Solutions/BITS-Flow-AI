<script lang="ts">
import {
	BookMarked,
	House,
	LayoutGrid,
	Settings as SettingsIcon,
	Sparkles,
	Users,
} from "@lucide/svelte";
import { onMount } from "svelte";
import { get } from "svelte/store";
import { m } from "./lib/paraglide/messages";
import { setLocale } from "./lib/paraglide/runtime";
import { activeMenu, initSystemListeners, loadAll, uiLocale } from "./lib/stores";
import type { MenuId } from "./lib/types";
import Accounts from "./lib/views/Accounts.svelte";
import Generate from "./lib/views/Generate.svelte";
import Home from "./lib/views/Home.svelte";
import Prompts from "./lib/views/Prompts.svelte";
import Providers from "./lib/views/Providers.svelte";
import SettingsView from "./lib/views/Settings.svelte";

type MenuEntry = { id: MenuId; icon: typeof House };

const menus: MenuEntry[] = [
	{ id: "home", icon: House },
	{ id: "accounts", icon: Users },
	{ id: "providers", icon: LayoutGrid },
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

onMount(() => {
	const stop = initSystemListeners();
	loadAll()
		.then(() => setLocale(get(uiLocale)))
		.catch((e) => console.error("init failed", e));
	return stop;
});
</script>

<div class="flex h-screen bg-bg text-fg">
  <nav class="w-[220px] shrink-0 border-r border-line bg-surface flex flex-col">
    <div class="h-12 flex items-center gap-2 px-4 border-b border-line">
      <span class="inline-block size-4 rounded-full bg-accent"></span>
      <span class="text-sm font-semibold tracking-wide">BITS Flow AI</span>
    </div>
    <ul class="p-2 space-y-0.5">
      {#key $uiLocale}
        {#each menus as entry (entry.id)}
          {@const Icon = entry.icon}
          <li>
            <button
              type="button"
              class="flex w-full items-center gap-2.5 rounded-md px-3 h-9 text-left transition-colors {$activeMenu === entry.id
                ? "bg-accent text-accent-fg font-medium"
                : "text-fg hover:bg-bg"}"
              onclick={() => activeMenu.set(entry.id)}
            >
              <Icon class="size-4 shrink-0" />
              <span class="truncate">{menuLabel[entry.id]()}</span>
            </button>
          </li>
        {/each}
      {/key}
    </ul>
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
