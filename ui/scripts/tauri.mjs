// Runs the Tauri CLI of this project with the app crate in ../app and the frontend in
// this folder, which the CLI would not find on its own (it expects src-tauri/).
// Usage: pnpm tauri dev | pnpm tauri build | pnpm tauri icon <png>
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const ui = fileURLToPath(new URL("..", import.meta.url));
const app = fileURLToPath(new URL("../../app", import.meta.url));
const cli = fileURLToPath(new URL("../node_modules/@tauri-apps/cli/tauri.js", import.meta.url));

const result = spawnSync(process.execPath, [cli, ...process.argv.slice(2)], {
  stdio: "inherit",
  env: { ...process.env, TAURI_APP_PATH: app, TAURI_FRONTEND_PATH: ui },
});
process.exit(result.status ?? 1);
