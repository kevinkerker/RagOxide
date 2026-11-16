// The frame "Antwort und Beleg" in the browser against the mock backend (Edge).
// SCREENSHOT_DIR=<folder> saves screenshots of the states compared with the design.
import { join } from "node:path";
import { expect, test, type Page } from "@playwright/test";
import chatDe from "../chat/i18n/de.json" with { type: "json" };
import chatEn from "../chat/i18n/en.json" with { type: "json" };
import settingsDe from "../settings/i18n/de.json" with { type: "json" };
import shellDe from "../shell/i18n/de.json" with { type: "json" };
import shellEn from "../shell/i18n/en.json" with { type: "json" };

const SCREENSHOT_DIR = process.env.SCREENSHOT_DIR;
const TEXT = {
  de: { shell: shellDe, chat: chatDe, example: /Wie oft wird/ },
  en: { shell: shellEn, chat: chatEn, example: /Wie oft wird/ },
};

async function shot(page: Page, name: string) {
  if (SCREENSHOT_DIR) await page.screenshot({ path: join(SCREENSHOT_DIR, `${name}.png`) });
}

/** Opens the example chat from the sidebar, or from the drawer in narrow layouts. */
async function openExampleChat(page: Page, language: "de" | "en" = "de") {
  const text = TEXT[language];
  const nav = page.getByRole("navigation", { name: text.shell.nav.label }).first();
  const inSidebar = nav.getByRole("button", { name: text.example });
  if (!(await inSidebar.isVisible())) {
    const opener = page.getByRole("button", { name: text.shell.nav.menu });
    if (await opener.isVisible()) await opener.click();
    else await nav.getByRole("button", { name: text.shell.nav.chats }).click();
    await page.getByRole("dialog").getByRole("button", { name: text.example }).click();
  } else {
    await inSidebar.click();
  }
  await expect(page.getByRole("heading", { level: 1, name: text.example })).toBeVisible();
}

function marker(page: Page, n: number, language: "de" | "en" = "de") {
  return page.getByRole("button", {
    name: TEXT[language].chat.marker.replace("{number}", String(n)),
  });
}

const SIZES = {
  wide: { width: 1440, height: 900 },
  narrow: { width: 560, height: 1560 },
};

for (const language of ["de", "en"] as const) {
  for (const scheme of ["light", "dark"] as const) {
    for (const size of ["wide", "narrow"] as const) {
      test(`answer and evidence (${language}, ${scheme}, ${size})`, async ({ page }) => {
        await page.emulateMedia({ colorScheme: scheme });
        await page.setViewportSize(SIZES[size]);
        await page.goto(`/?mock-lang=${language}`);
        await openExampleChat(page, language);
        await shot(page, `frame-no-document-${language}-${scheme}-${size}`);
        await marker(page, 2, language).click();
        await expect(marker(page, 2, language)).toHaveAttribute("aria-pressed", "true");
        await expect(page.locator("#evidence mark")).toBeVisible();
        await expect(page.locator("html")).toHaveAttribute("lang", language);
        const overflow = await page.evaluate(
          () => document.documentElement.scrollWidth - window.innerWidth,
        );
        expect(overflow).toBeLessThanOrEqual(0);
        await shot(page, `frame-${language}-${scheme}-${size}`);
      });
    }
  }
}

test("150 % scaling uses the rail and keeps answer and evidence side by side", async ({
  browser,
}) => {
  const context = await browser.newContext({
    viewport: { width: 960, height: 1040 },
    deviceScaleFactor: 1.5,
  });
  const page = await context.newPage();
  await page.goto("/");
  await openExampleChat(page);
  await marker(page, 2).click();
  await expect(page.locator("#evidence mark")).toBeVisible();
  const answer = await page.locator("#answer").boundingBox();
  const evidence = await page.locator("#evidence").boundingBox();
  expect(evidence!.x).toBeGreaterThan(answer!.x + answer!.width - 1);
  await shot(page, "frame-scale-150");
  await context.close();
});

test("the divider moves with keyboard and pointer, collapses and is remembered", async ({
  page,
}) => {
  await page.setViewportSize(SIZES.wide);
  await page.goto("/");
  await openExampleChat(page);
  const divider = page.getByRole("separator", { name: shellDe.evidence.divider });
  await expect(divider).toHaveAttribute("aria-valuenow", "600");
  await divider.focus();
  await page.keyboard.press("ArrowLeft");
  await expect(divider).toHaveAttribute("aria-valuenow", "616");
  await page.keyboard.press("Shift+ArrowRight");
  await expect(divider).toHaveAttribute("aria-valuenow", "552");
  await page.keyboard.press("End");
  await expect(divider).toHaveAttribute("aria-valuenow", "320");
  await page.keyboard.press("Home");
  const max = Number(await divider.getAttribute("aria-valuemax"));
  await expect(divider).toHaveAttribute("aria-valuenow", String(max));
  expect((await page.locator("#evidence").boundingBox())!.width).toBeCloseTo(max, 0);

  // Pointer: dragging 100px to the right makes the evidence column 100px narrower.
  await page.keyboard.press("End");
  const box = (await divider.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + 200);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 - 100, box.y + 200, { steps: 5 });
  await page.mouse.up();
  const dragged = Number(await divider.getAttribute("aria-valuenow"));
  expect(Math.abs(dragged - 420)).toBeLessThanOrEqual(4);

  // Enter collapses; the state survives a reload; a marker opens the column again.
  await divider.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator("#evidence")).toBeHidden();
  const expand = page.getByRole("button", { name: shellDe.evidence.label, exact: true });
  await expect(expand).toBeFocused();
  await shot(page, "frame-collapsed");
  await page.reload();
  await openExampleChat(page);
  await expect(page.locator("#evidence")).toBeHidden();
  await marker(page, 1).click();
  await expect(page.locator("#evidence")).toBeVisible();
  await expect(page.getByRole("separator")).toHaveAttribute("aria-valuenow", String(dragged));
});

test("without the answer model asking is locked, everything else works", async ({ page }) => {
  await page.setViewportSize(SIZES.wide);
  await page.goto("/?mock-first-run");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(settingsDe.setupTitle);
  await page.getByRole("button", { name: shellDe.nav.newChat }).click();
  const input = page.getByLabel(chatDe.inputLabel);
  await expect(input).toBeDisabled();
  await expect(page.getByText(shellDe.askLock.modelsMissing)).toBeVisible();
  await shot(page, "frame-locked");

  // Library stays usable during the setup.
  await page.getByRole("link", { name: shellDe.nav.library }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Bibliothek" })).toBeVisible();

  await page.getByRole("link", { name: shellDe.nav.setup }).click();
  await page.getByRole("button", { name: settingsDe.download }).click();
  const bar = page.getByRole("contentinfo", { name: shellDe.status.label });
  await expect(bar.getByRole("button", { name: /Modelle werden geladen: .* GB/ })).toBeVisible();
  await shot(page, "frame-status-download");
  await expect(page.getByText(settingsDe.ready)).toBeVisible({ timeout: 20_000 });
  await page.getByRole("button", { name: shellDe.nav.newChat }).click();
  await expect(page.getByLabel(chatDe.inputLabel)).toBeEnabled();
});

test("the status bar shows ingestion, errors and an unreachable folder", async ({ page }) => {
  await page.setViewportSize(SIZES.wide);
  await page.goto("/");
  const bar = page.getByRole("contentinfo", { name: shellDe.status.label });
  await expect(bar).toContainText("Bibliothek aktuell · 7 Dokumente");
  await shot(page, "frame-status-idle");
  // As the user does it: choose the folder in the dialog, then register it.
  const addPickedFolder = () =>
    page.evaluate(async () => {
      const mock = (globalThis as unknown as { __ragoxideMock: import("../api").MockBackend })
        .__ragoxideMock;
      const folder = await mock.api.pick_folder({ purpose: "watch", start: null });
      void mock.api.add_folder({ path: folder! });
    });
  await addPickedFolder();
  await expect(bar.getByText(/Liest ein: \d+ von \d+ Dateien/)).toBeVisible();
  await shot(page, "frame-status-ingest");
  await expect(bar.getByText("1 Datei mit Fehler")).toBeVisible({ timeout: 10_000 });
  await addPickedFolder();
  await expect(bar.getByText("\\\\nas\\Archiv nicht erreichbar")).toBeVisible();
  await shot(page, "frame-status-problems");
  await bar.getByText("1 Datei mit Fehler").click();
  await expect(page.getByRole("heading", { level: 1, name: "Bibliothek" })).toBeVisible();
});

test("Literata is embedded and loads without network", async ({ page }) => {
  const foreign: string[] = [];
  page.on("request", (request) => {
    const url = request.url();
    if (!url.startsWith("http://localhost:5173") && !/^(data|blob):/.test(url)) {
      foreign.push(url);
    }
  });
  await page.route(/^(?!http:\/\/localhost:5173)/, (route) => route.abort());
  await page.setViewportSize(SIZES.wide);
  await page.goto("/");
  await openExampleChat(page);
  const loaded = await page.evaluate(async () => {
    await document.fonts.ready;
    const faces = [...document.fonts].filter((f) => f.family.replace(/"/g, "") === "Literata");
    return {
      check: document.fonts.check('17px "Literata"'),
      statuses: faces.map((f) => `${f.style}:${f.status}`),
      family: getComputedStyle(document.querySelector("#answer .read")!).fontFamily,
    };
  });
  expect(loaded.check).toBe(true);
  expect(loaded.statuses).toContain("normal:loaded");
  expect(loaded.family).toMatch(/^"?Literata/);
  expect(foreign).toEqual([]);
});

// Colour scheme setting (ADR 0024): system follows the operating system, light and dark
// override it. The mock starts with the setting from ?mock-color-scheme.
const SCHEME_CASES = [
  { setting: "system", os: "light", expected: "light" },
  { setting: "system", os: "dark", expected: "dark" },
  { setting: "light", os: "dark", expected: "light" },
  { setting: "dark", os: "light", expected: "dark" },
] as const;

for (const { setting, os, expected } of SCHEME_CASES) {
  test(`colour scheme ${setting} on a ${os} system is ${expected}`, async ({ page }) => {
    await page.emulateMedia({ colorScheme: os });
    await page.setViewportSize(SIZES.wide);
    await page.goto(`/?mock-color-scheme=${setting}`);
    await expect(page.getByRole("button", { name: shellDe.nav.newChat })).toBeVisible();
    const colors = await page.evaluate(() => ({
      frame: getComputedStyle(document.body).backgroundColor,
      side: getComputedStyle(document.querySelector("nav")!).backgroundColor,
      text: getComputedStyle(document.body).color,
    }));
    expect(colors).toEqual(
      expected === "light"
        ? { frame: "rgb(243, 242, 238)", side: "rgb(235, 233, 227)", text: "rgb(27, 26, 23)" }
        : { frame: "rgb(25, 24, 22)", side: "rgb(30, 28, 26)", text: "rgb(236, 231, 223)" },
    );
    await shot(page, `scheme-${setting}-on-${os}-system`);
  });
}

test("scenarios from the URL: unreachable folder and failed ingestion", async ({ page }) => {
  await page.setViewportSize(SIZES.wide);
  await page.goto("/?mock-scenario=folder-unreachable,ingest-error");
  const bar = page.getByRole("contentinfo", { name: shellDe.status.label });
  await expect(bar.getByText("1 Datei mit Fehler")).toBeVisible();
  await expect(bar.getByText(/nas.Archiv nicht erreichbar/)).toBeVisible();
  await shot(page, "frame-scenario-unreachable");
});

test("an answer shows its phase, can be cancelled and stays in the chat", async ({ page }) => {
  await page.setViewportSize(SIZES.wide);
  await page.goto("/");
  await page.getByRole("button", { name: shellDe.nav.newChat }).click();
  // ADR 0054: first the search runs ...
  await page.getByLabel(chatDe.inputLabel).fill("Wie oft wird ein Gerät geprüft? mock:slow-search");
  await page.getByRole("button", { name: chatDe.send }).click();
  await expect(page.getByRole("status").filter({ hasText: chatDe.phase.searching })).toBeVisible();
  await page.getByRole("button", { name: chatDe.cancel }).click();
  await expect(page.getByText(chatDe.cancelled)).toBeVisible();
  // ... then, for a first question, the model starts. The chat holds only a cancelled
  // turn, so this is no follow-up.
  await page.getByLabel(chatDe.inputLabel).fill("Wie oft wird ein Gerät geprüft? mock:slow-start");
  await page.getByRole("button", { name: chatDe.send }).click();
  await expect(
    page.getByRole("status").filter({ hasText: chatDe.phase.starting_model }),
  ).toBeVisible();
  // The sources are already in the evidence column while the model starts (ADR 0043, 0054).
  await expect(
    page
      .locator("#evidence")
      .getByRole("button", { name: /pruefordnung/ })
      .first(),
  ).toBeVisible();
  await shot(page, "frame-answer-waiting");
  await page.getByRole("button", { name: chatDe.cancel }).click();
  await expect(page.getByText(chatDe.cancelled)).toBeVisible();
  await expect(page.locator("nav").getByRole("button", { name: /mock:slow/ })).toBeVisible();
});

test("a finished setup is remembered on the device (ADR 0048)", async ({ page }) => {
  await page.setViewportSize(SIZES.wide);
  // Models there (offline installer), setup not finished: the setup opens.
  await page.goto("/?mock-scenario=offline-installed");
  await expect(page).toHaveURL(/#\/setup$/);
  // A new start without finishing: the setup opens again; the mock keeps the flag as is.
  await page.goto("/?mock-scenario=offline-installed&mock-keep-setup");
  await expect(page).toHaveURL(/#\/setup$/);
  await expect(page.getByRole("link", { name: shellDe.nav.setup })).toBeVisible();
  await page.getByRole("button", { name: settingsDe.finish }).click();
  await expect(page.getByRole("link", { name: shellDe.nav.setup })).toHaveCount(0);
  // The same models and a new start: only what the shell remembered decides.
  await page.goto("/?mock-scenario=offline-installed&mock-keep-setup");
  await expect(page.getByRole("button", { name: shellDe.nav.newChat })).toBeVisible();
  await expect(page.getByRole("link", { name: shellDe.nav.setup })).toHaveCount(0);
  await expect(page).not.toHaveURL(/#\/setup$/);
});

test("the divider: 24px grab area and a line of at least 3:1 in light and dark", async ({
  page,
}) => {
  for (const scheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme: scheme });
    await page.setViewportSize(SIZES.wide);
    await page.goto("/");
    await openExampleChat(page);
    const divider = page.getByRole("separator", { name: shellDe.evidence.divider });
    expect((await divider.boundingBox())!.width).toBe(24);
    const ratio = await divider.evaluate((element) => {
      const parse = (color: string) => {
        const probe = document.createElement("canvas").getContext("2d")!;
        probe.fillStyle = color;
        probe.fillRect(0, 0, 1, 1);
        return [...probe.getImageData(0, 0, 1, 1).data.slice(0, 3)];
      };
      const luminance = (rgb: number[]) => {
        const [r, g, b] = rgb.map((v) => {
          const c = v / 255;
          return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
        });
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
      };
      const line = luminance(parse(getComputedStyle(element, "::after").backgroundColor));
      const ground = luminance(parse(getComputedStyle(document.body).backgroundColor));
      return (Math.max(line, ground) + 0.05) / (Math.min(line, ground) + 0.05);
    });
    expect(ratio).toBeGreaterThanOrEqual(3);
    await shot(page, `divider-${scheme}`);
  }
});

test("rendered states for the review: rail, setup in three widths, library", async ({ page }) => {
  const widths = { wide: 1440, medium: 960, narrow: 560 } as const;
  for (const [name, width] of Object.entries(widths)) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto("/?mock-scenario=first-run");
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(settingsDe.setupTitle);
    // Wide and medium show the setup entry at once; narrow in the drawer.
    if (name === "narrow") await page.getByRole("button", { name: shellDe.nav.menu }).click();
    const entry = page.getByRole("link", { name: new RegExp(shellDe.nav.setup) }).first();
    await expect(entry).toBeVisible();
    await expect(entry).toContainText(name === "medium" ? "2/3" : "Schritt 2 von 3");
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - window.innerWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
    await shot(page, `setup-${name}`);
  }
  // The rail in the medium layout, with a selected entry.
  await page.setViewportSize({ width: 960, height: 900 });
  await page.goto("/#/library");
  const rail = page.getByRole("navigation", { name: shellDe.nav.label });
  await expect(rail.getByRole("link", { name: shellDe.nav.library })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(rail.getByRole("button", { name: shellDe.nav.newChat })).toHaveText(
    shellDe.nav.newChat,
  );
  await shot(page, "rail-medium");
  // The library takes the full width of the main area.
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/#/library");
  const main = (await page.locator("main").boundingBox())!;
  const content = (await page.locator(".page-content").boundingBox())!;
  expect(content.width).toBeCloseTo(main.width, 0);
  await shot(page, "library-full-width");
});

test("answer levels: a level without its model offers the download (ADR 0058)", async ({
  page,
}) => {
  await page.setViewportSize(SIZES.wide);
  await page.goto("/");
  await page.getByLabel(chatDe.levelLabel).selectOption("balanced");
  await page.getByLabel(chatDe.inputLabel).fill("Wie oft wird ein Gerät geprüft?");
  await page.getByRole("button", { name: chatDe.send }).click();
  const alert = page.getByRole("alert");
  await expect(alert).toContainText(chatDe.level.balanced);
  await alert.getByRole("button", { name: chatDe.download }).click();
  const bar = page.getByRole("contentinfo", { name: shellDe.status.label });
  await expect(bar.getByRole("button", { name: /Modelle werden geladen: .* GB/ })).toBeVisible();
  await shot(page, "level-download");
  await expect(page.getByText(chatDe.modelReady)).toBeVisible({ timeout: 30_000 });
});

/**
 * Records every text of the status bar's download entry as the page changes it, with a
 * MutationObserver instead of sampling at fixed times, so a slow machine misses no state.
 * Returns the function that reads the texts so far.
 */
async function recordDownloadTexts(page: Page): Promise<() => Promise<string[]>> {
  await page.evaluate(() => {
    const texts: string[] = [];
    (globalThis as { __downloadTexts?: string[] }).__downloadTexts = texts;
    const take = () => {
      const text = document.querySelector('[data-status="download"]')?.textContent?.trim();
      if (text && texts.at(-1) !== text) texts.push(text);
    };
    new MutationObserver(take).observe(document.body, {
      subtree: true,
      childList: true,
      characterData: true,
    });
    take();
  });
  return () => page.evaluate(() => (globalThis as { __downloadTexts?: string[] }).__downloadTexts!);
}

/** Bytes done, size and percent of a download entry's text. */
function parseDownload(text: string) {
  const [, done, total, percent] =
    /([\d,]+) von ([\d,]+) GB · (\d+) %/.exec(text) ?? ([] as string[]);
  const number = (value: string) => Number(value.replace(",", "."));
  return { done: number(done), total: number(total), percent: Number(percent) };
}

/** Waits until `level` is ready in the shell the app uses (`__ragoxideShell`). */
async function levelReady(page: Page, level: string) {
  await page.waitForFunction(
    (wanted) =>
      (
        globalThis as unknown as { __ragoxideShell: import("../shell/setup.svelte").MockShell }
      ).__ragoxideShell.setup.level(wanted as "fast")?.ready === true,
    level,
    { timeout: 60_000 },
  );
}

test("after the setup, a higher level's download starts at 0 % and only rises (S08)", async ({
  page,
}) => {
  // The way of the review: first run, load the models, then "Ausgewogen" from the chat.
  await page.setViewportSize(SIZES.wide);
  await page.goto("/?mock-first-run");
  await page.getByRole("button", { name: settingsDe.download }).click();
  await expect(page.getByText(settingsDe.ready)).toBeVisible({ timeout: 60_000 });
  await page.getByRole("button", { name: settingsDe.finish }).click();
  await page.getByRole("button", { name: shellDe.nav.newChat }).click();
  await page.getByLabel(chatDe.levelLabel).selectOption("balanced");
  await page.getByLabel(chatDe.inputLabel).fill("Wie oft wird ein Gerät geprüft?");
  await page.getByRole("button", { name: chatDe.send }).click();
  const texts = await recordDownloadTexts(page);
  await page.getByRole("alert").getByRole("button", { name: chatDe.download }).click();
  await levelReady(page, "balanced");
  const seen = await texts();
  const percents = seen.map((text) => parseDownload(text).percent);
  expect(percents.length).toBeGreaterThan(1);
  expect(percents[0]).toBe(0);
  for (let i = 1; i < percents.length; i++) {
    expect(percents[i]).toBeGreaterThanOrEqual(percents[i - 1]);
  }
  expect(percents.at(-1)).toBe(100);
  // Always the size of the level, never "0,0 von 0,0 GB".
  expect(seen.every((text) => text.includes("von 5,1 GB"))).toBe(true);
});

test("Ausgewogen while the setup loads: one sum of about 9.5 GB, only rising (S08)", async ({
  page,
}) => {
  // The way of the fourth review: the setup loads, K starts Ausgewogen at the same time.
  // Ausgewogen starts at the setup's first downloaded bytes, in the page, so it joins
  // while the setup loads however fast the machine is.
  await page.setViewportSize(SIZES.wide);
  await page.goto("/?mock-first-run");
  await page.evaluate(async () => {
    const mock = (globalThis as unknown as { __ragoxideMock: import("../api").MockBackend })
      .__ragoxideMock;
    const shell = (
      globalThis as unknown as { __ragoxideShell: import("../shell/setup.svelte").MockShell }
    ).__ragoxideShell;
    let started = false;
    await mock.listen("model_progress", ({ level, event }) => {
      if (started || level !== "fast" || event.status.type !== "downloading") return;
      if (event.status.done_bytes === 0) return;
      started = true;
      void shell.setup.provide(null, "balanced");
    });
  });
  const texts = await recordDownloadTexts(page);
  await page.getByRole("button", { name: settingsDe.download }).click();
  await levelReady(page, "balanced");
  const both = (await texts()).map(parseDownload).filter((p) => p.total > 5);
  expect(both.length).toBeGreaterThan(1);
  // Base once plus E4B (9.53 GB) and the files without a size in models.toml.
  expect(both.every((p) => p.total >= 9.5 && p.total <= 9.6)).toBe(true);
  for (let i = 1; i < both.length; i++) {
    expect(both[i].total).toBeGreaterThanOrEqual(both[i - 1].total);
    expect(both[i].done).toBeGreaterThanOrEqual(both[i - 1].done);
    expect(both[i].percent).toBeGreaterThanOrEqual(both[i - 1].percent);
  }
  expect(both.at(-1)!.percent).toBe(100);
});

test("an offline package after a paused download continues its part (S08, contract of provide)", async ({
  page,
}) => {
  // The way of the fifth review: Ausgewogen from the chat, paused at about half, then the
  // offline package as L takes it over. By the contract of `ModelHub::provide` the copy
  // continues the partial file.
  await page.setViewportSize(SIZES.wide);
  await page.goto("/");
  // The mock holds the download once the level stands at 40 % or more, so "Laden
  // anhalten" is clicked at about half however fast the machine is (it was a race with
  // the download's end on a busy machine).
  await page.evaluate(async () => {
    const mock = (globalThis as unknown as { __ragoxideMock: import("../api").MockBackend })
      .__ragoxideMock;
    const shell = (
      globalThis as unknown as { __ragoxideShell: import("../shell/setup.svelte").MockShell }
    ).__ragoxideShell;
    let holding = false;
    await mock.listen("model_progress", () => {
      if (holding || (shell.setup.level("balanced")?.progress?.fraction ?? 0) < 0.4) return;
      holding = true;
      mock.modelFiles.hold();
    });
  });
  await page.getByLabel(chatDe.levelLabel).selectOption("balanced");
  await page.getByLabel(chatDe.inputLabel).fill("Wie oft wird ein Gerät geprüft?");
  await page.getByRole("button", { name: chatDe.send }).click();
  await page.getByRole("alert").getByRole("button", { name: chatDe.download }).click();
  const entry = page.locator('[data-status="download"]');
  await expect(entry).toHaveText(/· ([4-9]\d) %/, { timeout: 60_000 });
  const paused = parseDownload((await entry.textContent())!);
  await page.getByRole("button", { name: chatDe.cancelDownload }).click();
  await expect(entry).toBeHidden();
  const texts = await recordDownloadTexts(page);
  await page.evaluate(async () => {
    const mock = (globalThis as unknown as { __ragoxideMock: import("../api").MockBackend })
      .__ragoxideMock;
    mock.modelFiles.release();
    const folder = await mock.api.pick_folder({ purpose: "offline_package", start: null });
    const shell = (
      globalThis as unknown as { __ragoxideShell: import("../shell/setup.svelte").MockShell }
    ).__ragoxideShell;
    void shell.setup.provideFromPackage(folder!);
  });
  await levelReady(page, "balanced");
  const copy = (await texts()).map(parseDownload).filter((p) => p.total === 5.1);
  expect(copy.length).toBeGreaterThan(1);
  // Where the paused download stopped, not at 0 and not beyond, and only rising.
  expect(copy[0]).toEqual(paused);
  for (let i = 1; i < copy.length; i++) {
    expect(copy[i].percent).toBeGreaterThanOrEqual(copy[i - 1].percent);
  }
  expect(copy.at(-1)!.percent).toBe(100);
});

test("an exclusion dropped before the interface listened reaches the library", async ({ page }) => {
  await page.goto("/?mock-scenario=exclusion-removed-at-start#/library");
  await expect(page.getByText(String.raw`C:\Ablage\Alt`, { exact: false })).toBeVisible();
});
