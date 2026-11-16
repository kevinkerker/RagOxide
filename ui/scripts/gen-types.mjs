// Regenerates api/bindings.ts from the Rust types in ../app (cargo test with
// UPDATE_BINDINGS=1). The same test without the variable fails while the file is outdated.
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const result = spawnSync(
  "cargo",
  ["test", "--package", "ragoxide-app", "--test", "bindings", "--quiet"],
  { cwd: root, stdio: "inherit", env: { ...process.env, UPDATE_BINDINGS: "1" } },
);
process.exit(result.status ?? 1);
