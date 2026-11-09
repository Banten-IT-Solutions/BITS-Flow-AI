// Persona toolbar: provider + account quick switch, hosted above the
// provider page webview. Plain DOM, no framework.
import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  accountSchema,
  openPersona,
  personaState,
  providerSchema,
  switchPersonaProvider,
} from "./lib/api";

const personaId = (window as { __PERSONA_ID__?: string }).__PERSONA_ID__;
const providerSel = document.getElementById("provider") as HTMLSelectElement;
const accountSel = document.getElementById("account") as HTMLSelectElement;
const nameEl = document.getElementById("name") as HTMLElement;
const tileEl = document.getElementById("tile") as HTMLElement;

// Toolbar runs inside trusted app windows; ID comes from Rust init script.
if (typeof personaId === "string" && personaId.length > 0) {
  void init(personaId);
}

function initials(name: string): string {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => (w[0] ?? "").toUpperCase())
    .join("");
}

async function init(id: string): Promise<void> {
  const [accounts, providers, currentUrl] = await Promise.all([
    invoke<unknown[]>("list_accounts").then((a) => z.array(accountSchema).parse(a)),
    invoke<unknown[]>("list_providers").then((p) => z.array(providerSchema).parse(p)),
    personaState(id),
  ]);

  const account = accounts.find((a) => a.id === id);
  nameEl.textContent = account?.name ?? id;
  tileEl.textContent = initials(account?.name ?? id);

  for (const p of providers) {
    providerSel.add(new Option(p.name, p.id, false, p.url === currentUrl));
  }
  for (const a of accounts) {
    accountSel.add(new Option(a.name, a.id, false, a.id === id));
  }
  providerSel.disabled = false;
  accountSel.disabled = false;

  // Provider change: navigate this persona's page in-place (same profile).
  providerSel.onchange = () => {
    const p = providers.find((x) => x.id === providerSel.value);
    if (p) void switchPersonaProvider(id, p.url);
  };
  // Account change: reveal/launch that account's window on the current
  // provider (Rust skips navigate when URL already matches → no reload).
  accountSel.onchange = () => {
    const p = providers.find((x) => x.url === currentUrl) ?? providers[0];
    if (p) void openPersona(accountSel.value, p.url, p.engine);
  };
}
