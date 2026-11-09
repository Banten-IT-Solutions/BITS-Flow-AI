//! Integration check: Generate must mount and render provider cards.
//! Regression for svelte/e/props_invalid_value — binding a record member
//! that is still undefined killed the view silently (1.3.0..1.5.4).
//! Run: npm run build && node scripts/check-generate.mjs
//! Needs system Chrome (or CHROME=/path) and a GENERATE_FIXTURE dir with
//! settings/accounts/providers/prompts/recent JSON, as written by the app.
//! ponytail: puppeteer-core + system Chrome instead of vitest+jsdom;
//! swap to @testing-library/svelte if more component tests appear.

import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import puppeteer from "puppeteer-core";

const root = fileURLToPath(new URL("..", import.meta.url));
const fixture = process.env.GENERATE_FIXTURE ?? "/tmp/pcdata";
const port = 8123;

const data = (n) => JSON.parse(readFileSync(`${fixture}/${n}.json`, "utf8"));

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

await page.evaluateOnNewDocument(
  (payload) => {
    let cb = 1;
    window.__TAURI_INTERNALS__ = {
      transformCallback: () => String(cb++),
      invoke: (cmd) => {
        const map = {
          get_settings: payload.settings,
          list_accounts: payload.accounts,
          list_providers: payload.providers,
          list_prompts: payload.prompts,
          list_recent: payload.recent,
          list_open_personas: [],
        };
        if (cmd.startsWith("plugin:event|")) return Promise.resolve(null);
        return Promise.resolve(map[cmd] ?? null);
      },
    };
  },
  {
    settings: data("settings"),
    accounts: data("accounts"),
    providers: data("providers"),
    prompts: data("prompts"),
    recent: data("recent"),
  },
);

await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle2" });
await page.waitForFunction(
  () =>
    [...document.querySelectorAll("nav button")].some((b) => b.textContent?.includes("Hasilkan")),
  { timeout: 8000 },
);
await page.evaluate(() =>
  [...document.querySelectorAll("nav button")]
    .find((b) => b.textContent?.includes("Hasilkan"))
    ?.click(),
);
await new Promise((r) => setTimeout(r, 800));

const state = await page.evaluate(() => ({
  title: document.querySelector("main h1")?.textContent ?? "",
  cards: document.querySelectorAll("main .grid > div").length,
}));

await browser.close();
server.kill();

const visible = data("providers").filter(
  (p) => p.category === "image" || p.category === "both",
).length;
if (errors.length) throw new Error(`page errors: ${errors.join(" | ")}`);
if (state.title !== "Hasilkan") throw new Error(`wrong view: ${state.title}`);
if (state.cards !== visible) throw new Error(`cards ${state.cards} != providers ${visible}`);
console.log(`check-generate: ok (${state.cards} cards)`);
