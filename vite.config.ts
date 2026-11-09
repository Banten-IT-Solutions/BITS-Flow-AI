/// <reference types="vitest/config" />
import { paraglideVitePlugin } from "@inlang/paraglide-js";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    tailwindcss(),
    svelte(),
    paraglideVitePlugin({
      project: "./project.inlang",
      outdir: "./src/lib/paraglide",
      strategy: ["globalVariable", "baseLocale"],
      emitTsDeclarations: true,
      isServer: 'typeof window === "undefined"',
    }),
  ],

  // Settings → About reads this. npm_package_version is set by every npm run
  // script (vite build, tauri build), so it always matches package.json.
  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version ?? "0.0.0"),
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
