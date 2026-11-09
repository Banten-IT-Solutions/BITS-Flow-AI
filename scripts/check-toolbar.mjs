//! Integration check: persona toolbar must mount and wire its selects.
//! Regression guard for the multiwebview toolbar (toolbar.html).
//! Run: npm run build && node scripts/check-toolbar.mjs
//! Needs system Chrome (or CHROME=/path) and a GENERATE_FIXTURE dir with
//! accounts/providers JSON, as written by the app.
//! ponytail: puppeteer-core + system Chrome instead of vitest+jsdom;
//! swap to @testing-library if more DOM tests appear.

import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import puppeteer from "puppeteer-core";

const root = fileURLToPath(new URL("..", import.meta.url));
const fixture = process.env.GENERATE_FIXTURE ?? "/tmp/pcdata";
const port = 8124;

const data = (n) => JSON.parse(readFileSync(`${fixture}/${n}.json`, "utf8"));
const accounts = data("accounts");
const providers = data("providers").filter((p) => p.category !== "video");

const server = spawn("python3", ["-m", "http.server", String(port), "--directory", `${root}dist`], {
  stdio: "ignore",
});
// Wait until the server answers; a blind sleep races slow starts.
for (let up = false; !up; ) {
  await new Promise((r) => setTimeout(r, 200));
  up = await fetch(`http://localhost:${port}/`)
    .then((r) => r.ok)
    .catch(() => false);
}

const browser = await puppeteer.launch({
  executablePath: process.env.CHROME ?? "/usr/bin/google-chrome",
  headless: "new",
  args: ["--no-sandbox", "--disable-dev-shm-usage"],
});
const page = await browser.newPage();
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));

const calls = [];
await page.evaluateOnNewDocument(
  (payload) => {
    let cb = 1;
    window.__PERSONA_ID__ = payload.personaId;
    window.__TAURI_INTERNALS__ = {
      transformCallback: () => String(cb++),
      invoke: (cmd, args) => {
        const map = {
          list_accounts: payload.accounts,
          list_providers: payload.providers,
        };
        if (cmd === "persona_state") return Promise.resolve(payload.currentUrl);
        if (cmd.startsWith("plugin:event|")) return Promise.resolve(null);
        if (cmd === "switch_persona_provider" || cmd === "open_persona") {
          payload.calls.push({ cmd, args });
          return Promise.resolve({ label: "persona-x" });
        }
        return Promise.resolve(map[cmd] ?? null);
      },
    };
    window.__calls = payload.calls;
  },
  {
    personaId: accounts[0].id,
    accounts,
    providers,
    currentUrl: providers[0].url,
    calls,
  },
);

await page.goto(`http://localhost:${port}/toolbar.html`, { waitUntil: "networkidle2" });
await page.waitForFunction(() => !document.getElementById("provider")?.disabled, { timeout: 8000 });

const state = await page.evaluate(() => ({
  name: document.getElementById("name")?.textContent ?? "",
  tile: document.getElementById("tile")?.textContent ?? "",
  providers: document.getElementById("provider")?.options.length ?? 0,
  accounts: document.getElementById("account")?.options.length ?? 0,
  selectedProvider: document.getElementById("provider")?.value ?? "",
  selectedAccount: document.getElementById("account")?.value ?? "",
}));

// Switch provider → same-window navigate.
await page.select("#provider", providers[1].id);
await new Promise((r) => setTimeout(r, 300));
// Switch account → reveal/launch other persona window.
await page.select("#account", accounts[1].id);
await new Promise((r) => setTimeout(r, 300));

const switched = await page.evaluate(() => window.__calls.slice());

await browser.close();
server.kill();

if (errors.length) throw new Error(`page errors: ${errors.join(" | ")}`);
if (state.name !== accounts[0].name) throw new Error(`name ${state.name}`);
if (state.providers !== providers.length) throw new Error(`providers ${state.providers}`);
if (state.accounts !== accounts.length) throw new Error(`accounts ${state.accounts}`);
if (state.selectedProvider !== providers[0].id) throw new Error("current provider not preselected");
if (state.selectedAccount !== accounts[0].id) throw new Error("own account not preselected");
if (switched[0]?.cmd !== "switch_persona_provider")
  throw new Error(`provider switch: ${JSON.stringify(switched)}`);
if (switched[1]?.cmd !== "open_persona")
  throw new Error(`account switch: ${JSON.stringify(switched)}`);
console.log(`check-toolbar: ok (${state.providers} providers, ${state.accounts} accounts)`);
