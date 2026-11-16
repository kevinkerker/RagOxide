import path from "node:path";
import js from "@eslint/js";
import { defineConfig } from "eslint/config";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";

// Each area belongs to one package (plan, Phase 1). Areas use the shared code through
// $api and $shell and never import each other, so that they can be built in parallel.
const AREAS = ["chat", "library", "viewer", "settings"];
const ROOT = import.meta.dirname;
/** The aliases of vite.config.ts and the folders they stand for. */
const ALIASES = { $api: "api", $shell: "shell" };

/** The area folder `file` lies in, `null` outside every area. */
function areaOf(file) {
  const [first] = path.relative(ROOT, file).split(path.sep);
  return AREAS.includes(first) ? first : null;
}

/** The literal text of a module specifier, `null` if it is computed at runtime. */
function literal(node) {
  if (node?.type === "Literal" && typeof node.value === "string") return node.value;
  if (node?.type === "TemplateLiteral" && node.expressions.length === 0) {
    return node.quasis[0].value.cooked;
  }
  return null;
}

// `no-restricted-imports` sees only static imports. This rule covers the other ways in:
// `import("…")` and `import.meta.glob("…")` in an area must name a literal path that is
// a package, an alias ($api, $shell) or stays inside the area's own folder.
const dynamicBoundary = {
  meta: {
    type: "problem",
    messages: {
      computed: "Dynamic imports in an area need a literal path, so the boundary can be checked.",
      outside:
        "Dynamic imports and import.meta.glob stay inside the area; shared code comes through $api and $shell.",
    },
  },
  create(context) {
    const area = areaOf(context.filename);
    if (!area) return {};
    const check = (node, specifier) => {
      if (specifier === null) return context.report({ node, messageId: "computed" });
      const pattern = specifier.replace(/^!/, "");
      // Aliases resolve like Vite does; "$shell/../library/x" lands in another area.
      const alias = Object.keys(ALIASES).find((a) => pattern === a || pattern.startsWith(`${a}/`));
      if (alias) {
        const home = path.join(ROOT, ALIASES[alias]);
        const target = path.join(home, pattern.slice(alias.length));
        const shared = target === home || target.startsWith(home + path.sep);
        if (!shared && areaOf(target) !== area) context.report({ node, messageId: "outside" });
        return;
      }
      if (!pattern.startsWith(".") && !pattern.startsWith("/")) return;
      const target = pattern.startsWith("/")
        ? path.join(ROOT, pattern)
        : path.resolve(path.dirname(context.filename), pattern);
      // A wildcard in the first segment ("../*/x") is no area name and counts as outside.
      if (areaOf(target) !== area) context.report({ node, messageId: "outside" });
    };
    return {
      ImportExpression(node) {
        check(node, literal(node.source));
      },
      "CallExpression[callee.type='MemberExpression'][callee.object.type='MetaProperty'][callee.property.name=/^glob/]"(
        node,
      ) {
        const first = node.arguments[0];
        const patterns = first?.type === "ArrayExpression" ? first.elements : [first];
        for (const pattern of patterns) check(node, literal(pattern));
      },
    };
  },
};

const local = { rules: { "dynamic-boundary": dynamicBoundary } };

const areaBoundaries = AREAS.map((area) => ({
  files: [`${area}/**/*.ts`, `${area}/**/*.svelte`],
  plugins: { ragoxide: local },
  rules: {
    "ragoxide/dynamic-boundary": "error",
    "no-restricted-imports": [
      "error",
      {
        patterns: [
          {
            group: AREAS.filter((other) => other !== area).flatMap((other) => [
              `**/${other}`,
              `**/${other}/**`,
            ]),
            message: "Areas do not import each other; shared code belongs in api/ or shell/.",
          },
          {
            group: ["**/api", "**/api/**", "**/shell", "**/shell/**"],
            message: "Import shared code through $api and $shell.",
          },
        ],
      },
    ],
  },
}));

export default defineConfig(
  {
    ignores: ["dist/", "node_modules/", "playwright-report/", "test-results/", "api/bindings.ts"],
  },
  js.configs.recommended,
  ts.configs.recommended,
  svelte.configs.recommended,
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts"],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
        extraFileExtensions: [".svelte"],
      },
    },
  },
  areaBoundaries,
);
