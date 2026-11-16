import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";
import type { Plugin } from "vite";
import { defineConfig } from "vitest/config";

const dir = (path: string) => fileURLToPath(new URL(path, import.meta.url));

/**
 * The SIL OFL 1.1 asks that the licence travels with the font (ADR 0030): the build puts
 * shell/fonts/OFL.txt next to the font files as assets/OFL.txt, so the app bundle has it.
 */
function fontLicence(): Plugin {
  return {
    name: "ragoxide-font-licence",
    apply: "build",
    generateBundle() {
      this.emitFile({
        type: "asset",
        fileName: "assets/OFL.txt",
        source: readFileSync(dir("./shell/fonts/OFL.txt")),
      });
    },
  };
}

export default defineConfig({
  plugins: [svelte(), svelteTesting(), fontLicence()],
  resolve: {
    alias: {
      $api: dir("./api"),
      $shell: dir("./shell"),
    },
  },
  // The Tauri CLI expects a fixed port and must see compiler errors in its own output.
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    // The mock backend reads the fixtures in ../kb-core/testdata.
    fs: { allow: [dir("..")] },
  },
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    include: ["{api,shell,chat,library,viewer,settings}/**/*.test.ts"],
  },
});
