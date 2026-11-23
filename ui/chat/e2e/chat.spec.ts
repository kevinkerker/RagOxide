// The chat in the browser against the mock backend (Edge, ADR 0114): first question with
// phases up to a selected source, follow-up, cancel, a level without its model with the
// dialog, pause and resume in the menu, rename and delete, the narrow window, nothing that
// loads (KC24), the hit area of markers (KC29), keyboard (KC61), sizes and contrast
// (KC63), the three widths in light and dark (KC65), English (KC66).
// SCREENSHOT_DIR=<folder> also saves screenshots of every state.
import { join } from "node:path";
import { chromium, expect, test, type Page } from "@playwright/test";
import de from "../i18n/de.json" with { type: "json" };
import en from "../i18n/en.json" with { type: "json" };

const SCREENSHOT_DIR = process.env.SCREENSHOT_DIR;
const QUESTION = "Wie oft wird ein Gerät geprüft?";
/** A phase slowed by `mock:slow-<phase>` lasts about this long (300 steps of 40 ms). */
const SLOW_PHASE_MS = 12_000;
const WIDTHS = { wide: 1440, medium: 960, narrow: 560 } as const;
type Width = keyof typeof WIDTHS;

async function shot(page: Page, name: string) {
  if (SCREENSHOT_DIR) {
    await page.screenshot({ path: join(SCREENSHOT_DIR, `chat-${name}.png`), fullPage: true });
  }
}

const field = (page: Page) => page.getByLabel(de.inputLabel);
const stepper = (page: Page, level: "fast" | "balanced" | "thorough" = "fast") =>
  page.getByRole("button", { name: de.levelButton.replace("{level}", de.level[level]) });
const marker = (page: Page, n: number) =>
  page.getByRole("button", { name: de.marker.replace("{number}", String(n)), exact: true });
const phaseText = (page: Page) => page.locator("#answer .turn .phase");

async function ask(page: Page, question: string) {
  await field(page).fill(question);
  // A lost input fails here with its reason, not as a wait for a locked "Senden".
  await expect(field(page)).toHaveValue(question);
  await expect(page.getByRole("button", { name: de.send, exact: true })).toBeEnabled();
  await page.getByRole("button", { name: de.send, exact: true }).click();
}

/** Opens the drawer in the narrow layout, where the sidebar is behind the menu button. */
async function navigation(page: Page) {
  const menu = page.getByRole("button", { name: "Chats und Navigation" });
  if (await menu.isVisible()) {
    await menu.click();
    return page.getByRole("dialog");
  }
  return page.locator("nav").first();
}

async function newChat(page: Page) {
  const nav = await navigation(page);
  await nav.getByRole("button", { name: "Neuer Chat" }).first().click();
}

/** Opens a chat from the sidebar, or from the drawer in narrower windows. */
async function openChat(page: Page, title: string | RegExp) {
  const inSidebar = page.locator("nav").first().getByRole("button", { name: title });
  if (await inSidebar.isVisible()) await inSidebar.click();
  else {
    const menu = page.getByRole("button", { name: "Chats und Navigation" });
    if (await menu.isVisible()) await menu.click();
    else await page.locator("nav").first().getByRole("button", { name: "Chats" }).click();
    await page.getByRole("dialog").getByRole("button", { name: title }).click();
  }
  await expect(page.getByRole("heading", { level: 1, name: title })).toBeVisible();
}

/**
 * Lets the next question starting with "markdown:" be answered with `text` and the first
 * hits of the fixtures as sources, citing those numbered in `cited`. The mock is the
 * backend of the page (`__ragoxideMock`); the answer is not saved.
 */
async function answerWith(page: Page, text: string, cited: number[]) {
  await page.evaluate(
    ({ text, cited }) => {
      type Mock = import("../../api").MockBackend;
      const mock = (globalThis as unknown as { __ragoxideMock: Mock }).__ragoxideMock;
      const original = mock.api.ask;
      mock.api.ask = async (args, onEvent) => {
        if (!args.question.startsWith("markdown:")) return original(args, onEvent);
        const hits = await mock.api.search({
          text: "Wie oft wird ein Gerät geprüft?",
          limit: 3,
          documents: null,
          mode: "hybrid",
        });
        onEvent({ type: "status", phase: "searching", progress: null });
        onEvent({ type: "sources", hits });
        onEvent({ type: "status", phase: "writing", progress: null });
        onEvent({ type: "delta", text });
        const answer = {
          text,
          citations: cited.map((number) => ({ number, chunk: hits[number - 1].chunk })),
          no_sources: false,
          level: args.level,
          outcome: { type: "complete" as const },
        };
        onEvent({ type: "done", answer });
        return answer;
      };
    },
    { text, cited },
  );
}

test("first question with phases and a citation up to the selected source", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await newChat(page);
  await shot(page, "empty-wide-light");
  await ask(page, `${QUESTION} mock:slow-start`);
  // The question stands at once; Abbrechen replaces Senden.
  await expect(page.locator("#answer article").first()).toContainText(QUESTION);
  await expect(page.getByRole("button", { name: de.cancel, exact: true })).toBeVisible();
  await expect(phaseText(page)).toHaveText(de.phase.starting_model.replace("{level}", "Schnell"));
  await expect(page.getByRole("status").filter({ hasText: "Antwortmodell" })).toHaveCount(1);
  // The found sources already stand in the evidence column (ADR 0043, 0124).
  await expect(page.locator("#evidence h2")).toHaveText(/^Gefunden \(\d\) zu Frage 1$/);
  await expect(page.locator("#evidence")).toContainText(de.sources.foundNote);
  await expect(page.locator("#answer .turn .wait")).toContainText(/Schnell · läuft seit 0:\d\d/);
  await shot(page, "waiting-wide-light");
  await expect(marker(page, 1)).toBeVisible({ timeout: 20_000 });
  await expect(page.locator("#evidence h2")).toHaveText("Quellen (2) zu Frage 1");
  await marker(page, 1).click();
  await expect(marker(page, 1)).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#evidence mark")).toBeVisible();
  await expect(page.locator("#evidence")).toContainText(de.selected);
  await shot(page, "answer-wide-light");
});

test("follow-up: the search rewrites, sources follow after the model start", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await openChat(page, QUESTION);
  await ask(page, "Und wo steht das Ergebnis? mock:slow-search");
  await expect(phaseText(page)).toHaveText(de.phase.searching);
  await expect(page.locator("#answer .turn .wait")).toContainText(
    "Formuliert deine Folgefrage um und sucht",
  );
  await expect(page.locator("#evidence h2")).toHaveText("Quellen zu Frage 2");
  await expect(page.locator("#evidence")).toContainText(de.sources.followUpNote);
  await shot(page, "follow-up-wide-light");
  await expect(page.locator("#answer article")).toHaveCount(2);
  await expect(page.locator("#answer article").nth(1).locator("footer")).toBeVisible({
    timeout: 20_000,
  });
  await expect(page.locator("#answer article").nth(1)).toHaveClass(/active/);
  await expect(
    page
      .locator("#answer article")
      .first()
      .getByRole("button", { name: /Quellen zeigen/ }),
  ).toBeVisible();
});

test("cancel: the answer ends cancelled and stays after opening the chat again", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await openChat(page, QUESTION);
  await ask(page, `${QUESTION} mock:slow-read`);
  await expect(phaseText(page)).toHaveText(de.phase.reading_sources);
  await expect(page.locator("#answer .turn .progress")).toContainText("%");
  await shot(page, "reading-wide-light");
  await page.getByRole("button", { name: de.cancel, exact: true }).click();
  const cancelled = page.locator("#answer article").nth(1);
  await expect(cancelled).toContainText(de.cancelled);
  await expect(cancelled.getByRole("button", { name: de.again.button })).toBeVisible();
  await shot(page, "cancelled-wide-light");
  await openChat(page, /Budget/);
  await openChat(page, QUESTION);
  await expect(page.locator("#answer article").nth(1)).toContainText(de.cancelled);
});

test("failure and no source, in both colour schemes", async ({ page }) => {
  for (const scheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme: scheme });
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto("/");
    await openChat(page, QUESTION);
    await ask(page, `${QUESTION} mock:fail-write`);
    const failed = page.locator("#answer article").nth(1);
    await expect(failed).toContainText(de.failed);
    await failed.getByRole("button", { name: de.details }).click();
    await expect(failed.locator('[lang="en"]')).toContainText("generator error");
    await shot(page, `failed-wide-${scheme}`);
    await ask(page, "Was kostet die Kantine? mock:no-sources");
    const none = page.locator("#answer article").nth(2);
    await expect(none.getByRole("heading", { level: 3 })).toHaveText(
      "Keine passenden Quellen gefunden.",
    );
    await none.getByRole("button", { name: de.none.similar }).click();
    await expect(page.locator("#evidence h2")).toHaveText(/^Ähnliche Stellen \(\d\) zu Frage 3$/);
    await expect(page.locator("#evidence")).toContainText(de.sources.unverified);
    await shot(page, `no-sources-wide-${scheme}`);
  }
});

test("a level without its model: dialog, download, pause and resume in the menu", async ({
  page,
}) => {
  test.setTimeout(120_000);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  // Hold the download at about half so pausing is reached however fast the machine is.
  await page.evaluate(async () => {
    type Mock = import("../../api").MockBackend;
    type Shell = import("../../shell/setup.svelte").MockShell;
    const g = globalThis as unknown as {
      __ragoxideMock: Mock;
      __ragoxideShell: Shell;
      __held?: boolean;
    };
    await g.__ragoxideMock.listen("model_progress", () => {
      if (g.__held || (g.__ragoxideShell.setup.level("balanced")?.progress?.fraction ?? 0) < 0.3) {
        return;
      }
      g.__held = true;
      g.__ragoxideMock.modelFiles.hold();
    });
  });
  await field(page).fill(QUESTION);
  await stepper(page).click();
  await shot(page, "menu-wide-light");
  await page.getByRole("menuitemradio", { name: /Ausgewogen/ }).click();
  const dialog = page.getByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("button", { name: "Bei „Schnell“ bleiben" })).toBeFocused();
  await shot(page, "dialog-wide-light");
  await dialog.getByRole("button", { name: de.download.load, exact: true }).click();
  await expect(dialog).toBeHidden();
  await expect(field(page)).toHaveValue(QUESTION);
  // The stepper stays on Schnell; the help line shows the progress.
  await expect(page.locator("#question-help")).toContainText(/„Ausgewogen“ wird geladen: \d+ %/);
  await expect(stepper(page)).toBeVisible();
  await page.waitForFunction(() => (globalThis as { __held?: boolean }).__held === true);
  await stepper(page).click();
  await expect(page.getByRole("menuitemradio", { name: /Ausgewogen/ })).toContainText(
    /Wird geladen: \d+ %/,
  );
  await shot(page, "menu-loading-wide-light");
  await page.getByRole("menuitem", { name: de.cancelDownload }).click();
  await expect(page.locator("#question-help")).toContainText(/angehalten bei \d+ %/);
  await page.evaluate(() =>
    (
      globalThis as unknown as { __ragoxideMock: import("../../api").MockBackend }
    ).__ragoxideMock.modelFiles.release(),
  );
  await stepper(page).click();
  await expect(page.getByRole("menuitemradio", { name: /Ausgewogen/ })).toContainText(
    /Angehalten bei \d+ %/,
  );
  await shot(page, "menu-paused-wide-light");
  await page.getByRole("menuitem", { name: de.resumeDownload }).click();
  await expect(stepper(page, "balanced")).toBeVisible({ timeout: 60_000 });
  await expect(page.locator("#question-help")).toContainText("Ausgewogen: Antwort beginnt");
});

test("rename and delete a chat", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await openChat(page, "Wie hoch ist das Budget?");
  await page.getByRole("button", { name: de.rename.button }).click();
  await page.getByLabel(de.rename.label).fill("Budget 2026");
  await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", { level: 1, name: "Budget 2026" })).toBeVisible();
  const nav = page.locator("nav").first();
  await expect(nav.getByRole("button", { name: "Budget 2026" })).toBeVisible();
  await nav.getByRole("button", { name: de.history.actions }).click();
  await shot(page, "history-menu-wide-light");
  await page.getByRole("menuitem", { name: de.history.delete }).click();
  const dialog = page.getByRole("dialog", { name: "„Budget 2026“ löschen?" });
  await expect(dialog.getByRole("button", { name: de.delete.cancel })).toBeFocused();
  await shot(page, "delete-wide-light");
  await dialog.getByRole("button", { name: de.delete.confirm }).click();
  await expect(nav.getByRole("button", { name: "Budget 2026" })).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(de.newTitle);
});

test("narrow window: the input sits in the fixed strip and comes back with its text", async ({
  page,
}) => {
  await page.setViewportSize({ width: 560, height: 1000 });
  await page.goto("/");
  await openChat(page, QUESTION);
  const strip = page.locator(".composer-slot");
  await expect(strip.getByLabel(de.inputLabel)).toBeVisible();
  await field(page).fill("Noch nicht gesendet");
  await shot(page, "answer-narrow-light");
  await page.setViewportSize({ width: 1440, height: 900 });
  await expect(page.locator(".composer-slot")).toHaveCount(0);
  await expect(page.locator("#answer").getByLabel(de.inputLabel)).toHaveValue(
    "Noch nicht gesendet",
  );
});

test("KC24 nothing loads without a click, a link does not navigate", async ({ page }) => {
  const foreign: string[] = [];
  page.on("request", (request) => {
    const url = request.url();
    if (!url.startsWith("http://localhost:5173") && !/^(data|blob):/.test(url)) foreign.push(url);
  });
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await newChat(page);
  await answerWith(
    page,
    'Ein Bild ![Plan](https://example.invalid/plan.png) und <img src="https://example.invalid/x.png">, ' +
      "ein [Link](https://example.invalid/seite) und <https://example.invalid/auto> [1].",
    [1],
  );
  await ask(page, "markdown: Bilder und Links");
  const article = page.locator("#answer article");
  await expect(article).toContainText("[Bild: Plan]");
  await expect(article.locator("img, a, iframe")).toHaveCount(0);
  const url = page.url();
  // A link is its text and its address as plain text (ADR 0139).
  await expect(article).toContainText("Link (https://example.invalid/seite)");
  await article.locator(".link").first().click();
  await article.getByText("https://example.invalid/auto").first().click();
  await page.waitForTimeout(300);
  expect(page.url()).toBe(url);
  expect(foreign).toEqual([]);
});

test("KC29 markers: 44px high hit area, 26 to 30px wide in groups, without overlap", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await newChat(page);
  await answerWith(page, "Ein Satz mit zwei Belegen [1][2]. Und einer allein [3].", [1, 2, 3]);
  await ask(page, "markdown: Marken");
  await expect(marker(page, 3)).toBeVisible();
  const measured = await page.evaluate(() => {
    const extent = (button: HTMLElement) => {
      const box = button.getBoundingClientRect();
      const y = box.top + box.height / 2;
      const x = box.left + box.width / 2;
      const hits = (px: number, py: number) => {
        const at = document.elementFromPoint(px, py);
        return at === button || button.contains(at);
      };
      let left = x;
      while (hits(left - 1, y)) left -= 1;
      let right = x;
      while (hits(right + 1, y)) right += 1;
      let top = y;
      while (hits(x, top - 1)) top -= 1;
      let bottom = y;
      while (hits(x, bottom + 1)) bottom += 1;
      return { left, right, width: right - left + 1, height: bottom - top + 1 };
    };
    const buttons = [...document.querySelectorAll<HTMLElement>("#answer .marker")];
    const paragraph = buttons[0].closest("p")!;
    return {
      marks: buttons.map(extent),
      lineHeight: getComputedStyle(paragraph).lineHeight,
      lines: Math.round(
        paragraph.getBoundingClientRect().height /
          parseFloat(getComputedStyle(paragraph).lineHeight),
      ),
      height: paragraph.getBoundingClientRect().height,
    };
  });
  const [first, second, single] = measured.marks;
  for (const mark of measured.marks) expect(mark.height).toBeGreaterThanOrEqual(44);
  expect(first.width).toBeGreaterThanOrEqual(26);
  expect(first.width).toBeLessThanOrEqual(30);
  expect(second.width).toBeGreaterThanOrEqual(26);
  expect(second.width).toBeLessThanOrEqual(30);
  expect(second.left).toBeGreaterThan(first.right);
  expect(single.width).toBeGreaterThanOrEqual(38);
  // The line spacing stays that of the reading text.
  expect(measured.lineHeight).toBe("30px");
  expect(measured.height).toBe(measured.lines * 30);
});

test("KC61 keyboard: input, stepper menu, send, markers and sources in reading order", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await openChat(page, QUESTION);
  await field(page).focus();
  await page.keyboard.press("Tab");
  await expect(stepper(page)).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("menuitemradio", { name: /Schnell/ })).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await expect(page.getByRole("menuitemradio", { name: /Ausgewogen/ })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("menu")).toHaveCount(0);
  await expect(stepper(page)).toBeFocused();
  // The focus ring of the shell is visible (2px).
  const outline = await stepper(page).evaluate((el) => getComputedStyle(el).outlineWidth);
  expect(outline).toBe("2px");
  // Markers: Tab reaches them, Enter and Space select.
  await marker(page, 1).focus();
  await page.keyboard.press("Enter");
  await expect(marker(page, 1)).toHaveAttribute("aria-pressed", "true");
  await marker(page, 2).focus();
  await page.keyboard.press("Space");
  await expect(marker(page, 2)).toHaveAttribute("aria-pressed", "true");
  // Selected and focused look different: filled number, and a ring only when focused.
  const looks = await page.evaluate(() => {
    const [one, two] = [...document.querySelectorAll<HTMLElement>("#answer .marker")];
    return {
      selectedBackground: getComputedStyle(two).backgroundColor,
      otherBackground: getComputedStyle(one).backgroundColor,
      focusedOutline: getComputedStyle(two).outlineStyle,
      otherOutline: getComputedStyle(one).outlineStyle,
    };
  });
  expect(looks.selectedBackground).not.toBe(looks.otherBackground);
  expect(looks.focusedOutline).toBe("solid");
  expect(looks.otherOutline).toBe("none");
  // Send with Enter from the field.
  await field(page).fill("Und wer unterschreibt?");
  await page.keyboard.press("Enter");
  await expect(page.locator("#answer article")).toHaveCount(2);
});

/** Contrast of texts (4.5:1) and of control lines (3:1) in the chat's places. */
async function contrastProblems(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const canvas = document.createElement("canvas").getContext("2d", {
      willReadFrequently: true,
    })!;
    const rgba = (color: string) => {
      canvas.clearRect(0, 0, 1, 1);
      canvas.fillStyle = "rgba(0,0,0,0)";
      canvas.fillStyle = color;
      canvas.fillRect(0, 0, 1, 1);
      return [...canvas.getImageData(0, 0, 1, 1).data];
    };
    const luminance = ([r, g, b]: number[]) => {
      const [R, G, B] = [r, g, b].map((v) => {
        const c = v / 255;
        return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
      });
      return 0.2126 * R + 0.7152 * G + 0.0722 * B;
    };
    const ratio = (a: number[], b: number[]) => {
      const [x, y] = [luminance(a), luminance(b)];
      return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
    };
    const ground = (element: Element | null): number[] => {
      for (let node = element; node; node = node.parentElement) {
        const color = rgba(getComputedStyle(node).backgroundColor);
        if (color[3] > 0) return color;
      }
      return rgba(getComputedStyle(document.body).backgroundColor);
    };
    const roots = [
      ...document.querySelectorAll(
        "#answer, #evidence .head, #evidence ul, nav, .composer, dialog, [role=menu]",
      ),
    ];
    const problems: string[] = [];
    for (const root of roots) {
      for (const element of root.querySelectorAll<HTMLElement>("*")) {
        const style = getComputedStyle(element);
        const box = element.getBoundingClientRect();
        if (box.width <= 1 || box.height <= 1 || style.visibility === "hidden") continue;
        if ((element as HTMLButtonElement).disabled) continue;
        const text = [...element.childNodes].some(
          (n) => n.nodeType === Node.TEXT_NODE && n.textContent!.trim() !== "",
        );
        if (text) {
          const value = ratio(rgba(style.color), ground(element));
          if (value < 4.5)
            problems.push(`text ${value.toFixed(2)} ${element.textContent!.trim().slice(0, 40)}`);
        }
        const isControl = element.matches("button, textarea, input, [role^=menuitem]");
        const border = parseFloat(style.borderTopWidth);
        const borderColor = rgba(style.borderTopColor);
        if (isControl && border >= 1 && style.borderTopStyle !== "none" && borderColor[3] > 0) {
          const behind = ground(element.parentElement);
          const own = rgba(style.backgroundColor);
          // A filled control is told apart by its fill or its border.
          const value = Math.max(ratio(borderColor, behind), own[3] > 0 ? ratio(own, behind) : 0);
          if (value < 3)
            problems.push(`line ${value.toFixed(2)} ${element.textContent!.trim().slice(0, 40)}`);
        }
      }
    }
    return problems;
  });
}

/** Controls of the chat smaller than 44 x 44px (markers excepted, KC29). */
async function smallControls(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    [
      ...document.querySelectorAll<HTMLElement>(
        "#answer button, #answer textarea, .composer button, .composer textarea, #evidence ul button, nav li button, dialog button, [role=menu] button",
      ),
    ]
      .filter((el) => !el.classList.contains("marker") && el.getBoundingClientRect().width > 0)
      .filter((el) => {
        const box = el.getBoundingClientRect();
        return box.height < 44 || box.width < 44;
      })
      .map(
        (el) =>
          `${el.textContent?.trim().slice(0, 30)} ${el.getBoundingClientRect().width}x${el.getBoundingClientRect().height}`,
      ),
  );
}

/** Nothing of the answer column scrolls sideways; menus and dialogs stay in the window. */
async function fits(page: Page) {
  return page.evaluate(() => {
    const out = (el: Element) => {
      const box = el.getBoundingClientRect();
      return (
        box.left < 0 ||
        box.top < 0 ||
        box.right > innerWidth + 0.5 ||
        box.bottom > innerHeight + 0.5
      );
    };
    const thread = document.querySelector("#answer .thread")!;
    return {
      page: document.documentElement.scrollWidth - innerWidth,
      answer: thread.scrollWidth - thread.clientWidth,
      outside: [...document.querySelectorAll("dialog[open], [role=menu]")].filter(out).length,
    };
  });
}

for (const scheme of ["light", "dark"] as const) {
  for (const width of Object.keys(WIDTHS) as Width[]) {
    test(`KC63 KC65 every state fits and has enough contrast (${width}, ${scheme})`, async ({
      page,
    }) => {
      await page.emulateMedia({ colorScheme: scheme });
      await page.setViewportSize({ width: WIDTHS[width], height: width === "narrow" ? 1100 : 900 });
      await page.goto("/");
      const check = async (state: string) => {
        expect(await contrastProblems(page), state).toEqual([]);
        expect(await smallControls(page), state).toEqual([]);
        const fit = await fits(page);
        expect(fit.page, state).toBeLessThanOrEqual(0);
        expect(fit.answer, state).toBeLessThanOrEqual(0);
        expect(fit.outside, state).toBe(0);
        await shot(page, `${state}-${width}-${scheme}`);
      };
      await newChat(page);
      await check("empty");
      await answerWith(
        page,
        "**Prüffristen** im Überblick [1]:\n\n- jährlich [2]\n- nach Mängeln *vier Wochen* [3]\n\n| Gerät | Frist | Ergebnis | Prüfer | Bemerkung |\n|---|---|---|---|---|\n| Kühlschrank Süd | jährlich [1] | bestanden | Werkstatt | ohne Befund, nächste Prüfung im November |\n| Wärmepumpe Ost | vier Wochen [2] | Mängel | Werkstatt | Dichtung tauschen, danach erneut prüfen |",
        [1, 2, 3],
      );
      await ask(page, "markdown: Übersicht");
      await expect(marker(page, 3).first()).toBeVisible();
      await marker(page, 2).first().click();
      await check("markdown");
      await openChat(page, QUESTION);
      await ask(page, `${QUESTION} mock:slow-read`);
      await expect(phaseText(page)).toHaveText(de.phase.reading_sources);
      await check("waiting");
      await page.getByRole("button", { name: de.cancel, exact: true }).click();
      await expect(page.locator("#answer article").nth(1)).toContainText(de.cancelled);
      await check("cancelled");
      await ask(page, `${QUESTION} mock:fail-read`);
      await expect(page.locator("#answer article").nth(2)).toContainText(de.failed);
      await check("failed");
      await page
        .locator("#answer article")
        .nth(2)
        .getByRole("button", { name: de.again.button })
        .click();
      await check("again-menu");
      await page.keyboard.press("Escape");
      await stepper(page).click();
      await check("level-menu");
      await page.getByRole("menuitemradio", { name: /Gründlich/ }).click();
      await check("dialog");
      await page.keyboard.press("Escape");
      await page.goto("/?mock-scenario=stale-sources");
      await page.evaluate(async () => {
        const mock = (globalThis as unknown as { __ragoxideMock: import("../../api").MockBackend })
          .__ragoxideMock;
        await mock.api.sync({ op: "s", mode: "normal" });
      });
      await openChat(page, /Notizen zur Prüfung/);
      await marker(page, 1).click();
      await check("stale");
    });
  }
}

test("KC66 English with ?mock-lang=en", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/?mock-lang=en");
  await page.getByRole("button", { name: "New chat" }).first().click();
  await expect(page.getByRole("heading", { level: 2, name: en.empty.title })).toBeVisible();
  await expect(page.getByLabel(en.inputLabel)).toBeVisible();
  await expect(page.getByRole("button", { name: "Level: Fast" })).toBeVisible();
  await expect(page.locator("#question-help")).toHaveText(en.help.price.fast);
  await page.getByLabel(en.inputLabel).fill(`${QUESTION} mock:slow-start`);
  await page.getByRole("button", { name: en.send, exact: true }).click();
  await expect(phaseText(page)).toHaveText(en.phase.starting_model.replace("{level}", "Fast"));
  await expect(page.locator("#evidence h2")).toHaveText(/^Found \(\d\) for question 1$/);
  await shot(page, "waiting-wide-light-en");
  await expect(page.locator("#answer article footer")).toContainText("Fast · 2 sources", {
    timeout: 20_000,
  });
  const german = await page.evaluate(() =>
    [...document.querySelectorAll("#answer, #evidence .head, #evidence ul, nav")]
      .map((el) => el.textContent ?? "")
      .join(" ")
      .match(/\b(Frage|Quelle|Senden|Abbrechen|Stufe|Heute|Gestern|Älter)\b/g),
  );
  expect(german).toBeNull();
});

// ---------------------------------------------------------------- rework after the first review

/** Takes a screenshot of the current state in wide and narrow, light and dark. */
async function shots(page: Page, name: string) {
  // Without a folder nothing is taken and the window keeps its size: switching to 560px
  // and back makes the shell's portal move the input, and a test typing at that moment
  // loses its text (KC83).
  if (!SCREENSHOT_DIR) return;
  const size = page.viewportSize()!;
  for (const [width, label] of [
    [1440, "wide"],
    [560, "narrow"],
  ] as const) {
    for (const scheme of ["light", "dark"] as const) {
      await page.setViewportSize({ width, height: width === 560 ? 1100 : 900 });
      await page.emulateMedia({ colorScheme: scheme });
      await page.waitForTimeout(150);
      await shot(page, `${name}-${label}-${scheme}`);
    }
  }
  await page.setViewportSize(size);
  await page.emulateMedia({ colorScheme: "light" });
  // Back in the test's layout (the resize arrives with the next rendering step).
  await page.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))),
  );
}

const LONG_ADDRESS =
  "https://intranet.example.invalid/sehr/langer/pfad/ohne/leerzeichen/dokument-version-2026-11-20-final.pdf";

for (const width of Object.keys(WIDTHS) as Width[]) {
  test(`KC65 KC76 a long address right before a marker and a link with its address (${width})`, async ({
    page,
  }) => {
    for (const scheme of ["light", "dark"] as const) {
      await page.emulateMedia({ colorScheme: scheme });
      await page.setViewportSize({ width: WIDTHS[width], height: 900 });
      await page.goto("/");
      await newChat(page);
      await answerWith(
        page,
        `Siehe ${LONG_ADDRESS}[1]. Mehr steht in der [Prüfordnung](${LONG_ADDRESS}) [2].`,
        [1, 2],
      );
      await ask(page, "markdown: lange Adresse");
      await expect(marker(page, 2)).toBeVisible();
      const article = page.locator("#answer article");
      await expect(article).toContainText(`Prüfordnung (${LONG_ADDRESS})`);
      await expect(article.locator("a")).toHaveCount(0);
      const measured = await page.evaluate(() => {
        const thread = document.querySelector("#answer .thread")!;
        const answer = document.querySelector("#answer")!.getBoundingClientRect();
        const markers = [...document.querySelectorAll("#answer .marker")].map((m) =>
          m.getBoundingClientRect(),
        );
        return {
          page: document.documentElement.scrollWidth - innerWidth,
          thread: thread.scrollWidth - thread.clientWidth,
          inside: markers.every((m) => m.left >= answer.left && m.right <= answer.right + 0.5),
        };
      });
      expect(measured.page).toBeLessThanOrEqual(0);
      expect(measured.thread).toBeLessThanOrEqual(0);
      expect(measured.inside).toBe(true);
      await shot(page, `long-address-${width}-${scheme}`);
    }
  });
}

/**
 * Lets the next question starting with "stream:" be answered with `count` paragraphs, one
 * delta every `ms`, longer than the window.
 */
async function streamWith(page: Page, count: number, ms: number) {
  await page.evaluate(
    ({ count, ms }) => {
      type Mock = import("../../api").MockBackend;
      const mock = (globalThis as unknown as { __ragoxideMock: Mock }).__ragoxideMock;
      const original = mock.api.ask;
      mock.api.ask = async (args, onEvent) => {
        if (!args.question.startsWith("stream:")) return original(args, onEvent);
        const hits = await mock.api.search({
          text: "Wie oft wird ein Gerät geprüft?",
          limit: 3,
          documents: null,
          mode: "hybrid",
        });
        onEvent({ type: "status", phase: "searching", progress: null });
        onEvent({ type: "sources", hits });
        onEvent({ type: "status", phase: "writing", progress: null });
        let text = "";
        for (let i = 1; i <= count; i++) {
          await new Promise((resolve) => setTimeout(resolve, ms));
          const piece = `Absatz ${i}: Jedes Gerät wird mindestens einmal jährlich geprüft, und das Ergebnis steht im Prüfprotokoll [1].\n\n`;
          text += piece;
          onEvent({ type: "delta", text: piece });
        }
        const answer = {
          text: text.trim(),
          citations: [{ number: 1, chunk: hits[0].chunk }],
          no_sources: false,
          level: args.level,
          outcome: { type: "complete" as const },
        };
        onEvent({ type: "done", answer });
        return answer;
      };
    },
    { count, ms },
  );
}

/**
 * How far the end of the thread lies below what is visible of it, the scroll position.
 * Measured after a rendering step (two animation frames), so that the `ResizeObserver`
 * that follows the thread has seen the last growth (KC83).
 */
async function threadEnd(page: Page) {
  return page.evaluate(async () => {
    await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    // The element that scrolls the thread: the thread, or the page in the narrow window.
    let box = document.querySelector<HTMLElement>("#answer .thread");
    while (box && !["auto", "scroll"].includes(getComputedStyle(box).overflowY)) {
      box = box.parentElement;
    }
    let bottom = box!.getBoundingClientRect().bottom;
    const strip = document.querySelector<HTMLElement>(".composer-slot");
    if (strip && strip.offsetHeight > 0)
      bottom = Math.min(bottom, strip.getBoundingClientRect().top);
    const flow = document.querySelector("#answer .flow")!;
    return {
      distance: flow.getBoundingClientRect().bottom - bottom,
      scrollTop: box!.scrollTop,
      behavior: getComputedStyle(box!).scrollBehavior,
    };
  });
}

/**
 * The view is at the end of the thread (at most 48px from it), waited for with repeated
 * measurements for about 1 s: a measurement between a growth and the next rendering step
 * sees one paragraph not yet followed (KC83). Following that comes later than that (the
 * 5 s a small step is kept, a delayed `follow`) fails.
 */
async function atEnd(page: Page, message?: string) {
  await expect
    .poll(async () => (await threadEnd(page)).distance, {
      message,
      timeout: 1_000,
      intervals: [50, 100, 100, 200],
    })
    .toBeLessThanOrEqual(48);
}

/**
 * Measures the distance to the end every 10 ms for `ms` while the text grows: following,
 * it never lies more than one paragraph behind (KC83). Returns the largest distance and
 * the height of one paragraph of the last answer (top to top).
 */
async function behindWhileGrowing(page: Page, ms: number) {
  return page.evaluate(async (ms) => {
    let box = document.querySelector<HTMLElement>("#answer .thread");
    while (box && !["auto", "scroll"].includes(getComputedStyle(box).overflowY)) {
      box = box.parentElement;
    }
    const distance = () => {
      let bottom = box!.getBoundingClientRect().bottom;
      const strip = document.querySelector<HTMLElement>(".composer-slot");
      if (strip && strip.offsetHeight > 0)
        bottom = Math.min(bottom, strip.getBoundingClientRect().top);
      return document.querySelector("#answer .flow")!.getBoundingClientRect().bottom - bottom;
    };
    let max = -Infinity;
    let min = Infinity;
    const start = performance.now();
    while (performance.now() - start < ms) {
      // Below 0 the end is in view (narrow, the page scrolls on below the thread).
      const d = Math.max(0, distance());
      max = Math.max(max, d);
      min = Math.min(min, d);
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    const articles = document.querySelectorAll("#answer article");
    const tops = [...articles[articles.length - 1].querySelectorAll(".markdown > p")].map(
      (p) => p.getBoundingClientRect().top,
    );
    let paragraph = 0;
    for (let i = 1; i < tops.length; i++) paragraph = Math.max(paragraph, tops[i] - tops[i - 1]);
    return { max, min, paragraph };
  }, ms);
}

/**
 * Following while the text grows: the distance never lies more than one paragraph above
 * its smallest value, and that is at the end or a kept small step (at most 48px).
 */
function expectFollowing(behind: { max: number; min: number; paragraph: number }, label?: string) {
  expect(behind.paragraph, label).toBeGreaterThan(48);
  expect(behind.min, label).toBeLessThanOrEqual(48);
  expect(behind.max - behind.min, label).toBeLessThanOrEqual(behind.paragraph + 1);
}

/**
 * Scrolls as the user does, with the mouse wheel over the thread: by `by` pixels, or
 * far enough down to reach its end. A programmatic scroll is no movement of the user.
 */
async function scrollThread(page: Page, by: number | "end", wait = 300) {
  // A point in the visible part of the thread, above the fixed strip of the narrow window.
  const point = await page.evaluate(() => {
    const flow = document.querySelector("#answer .flow")!.getBoundingClientRect();
    let box = document.querySelector<HTMLElement>("#answer .thread");
    while (box && !["auto", "scroll"].includes(getComputedStyle(box).overflowY)) {
      box = box.parentElement;
    }
    const strip = document.querySelector<HTMLElement>(".composer-slot");
    let bottom = Math.min(flow.bottom, box!.getBoundingClientRect().bottom);
    if (strip && strip.offsetHeight > 0)
      bottom = Math.min(bottom, strip.getBoundingClientRect().top);
    return { x: flow.left + flow.width / 2, y: Math.max(flow.top, bottom - 120) };
  });
  await page.mouse.move(point.x, point.y);
  await page.mouse.wheel(0, by === "end" ? 100_000 : by);
  await page.waitForTimeout(wait);
}

/** Scrolls with a key, the focus in the thread (as on a marker reached with Tab). */
async function scrollWithKey(page: Page, key: string) {
  await page.evaluate(() =>
    document.querySelector<HTMLElement>("#answer .turn .marker")?.focus({ preventScroll: true }),
  );
  await page.keyboard.press(key);
  await page.waitForTimeout(400);
}

for (const width of Object.keys(WIDTHS) as Width[]) {
  test(`KC78 KC79a the thread follows a growing answer at its end, not when scrolled up (${width})`, async ({
    page,
  }) => {
    test.setTimeout(60_000);
    await page.setViewportSize({ width: WIDTHS[width], height: 800 });
    await page.goto("/");
    await newChat(page);
    await streamWith(page, 60, 120);
    await ask(page, "stream: lange Antwort");
    const article = page.locator("#answer article");
    await expect(article).toContainText("Absatz 12:");
    // At the end, it follows, never more than one paragraph behind; no animation.
    await atEnd(page);
    expectFollowing(await behindWhileGrowing(page, 1500));
    let end = await threadEnd(page);
    expect(end.behavior).toBe("auto");
    // The resting caret: drawn after the last paragraph, without animation (KC79 a).
    const caret = await page.evaluate(() => {
      const last = document.querySelector("#answer .markdown.caret > :last-child")!;
      const style = getComputedStyle(last, "::after");
      return { content: style.content, animation: style.animationName };
    });
    expect(caret.content).toBe('""');
    expect(caret.animation).toBe("none");
    await shot(page, `follow-${width}`);
    // Scrolled up: it stays while the text grows.
    await scrollThread(page, -400);
    const stopped = await threadEnd(page);
    await expect(article).toContainText("Absatz 25:");
    end = await threadEnd(page);
    expect(end.scrollTop).toBe(stopped.scrollTop);
    expect(end.distance).toBeGreaterThan(48);
    // Back at the end: it follows again, up to the footer after `done`.
    await scrollThread(page, "end");
    await expect(article).toContainText("Absatz 35:");
    await atEnd(page);
    expectFollowing(await behindWhileGrowing(page, 1000));
    await expect(article.locator("footer")).toBeVisible({ timeout: 20_000 });
    await page.waitForTimeout(200);
    await atEnd(page);
    await expect(page.locator("#answer .markdown.caret")).toHaveCount(0);
    await shot(page, `follow-done-${width}`);
  });
}

test("KC79a the caret while text comes in, gone after a cancel", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await newChat(page);
  await ask(page, `${QUESTION} mock:slow-write`);
  await expect(page.locator("#answer .markdown.caret")).toBeVisible({ timeout: 10_000 });
  await shots(page, "caret");
  await page.getByRole("button", { name: de.cancel, exact: true }).click();
  await expect(page.locator("#answer article")).toContainText(de.cancelled);
  await expect(page.locator("#answer .markdown.caret")).toHaveCount(0);
});

test("KC75 Neuer Chat while the first answer runs: empty chat, the running one in the history, both saved", async ({
  page,
}) => {
  // The waits follow the control words, not a fixed speed of the machine (KC83): the
  // first answer reads and writes slowly, so it still runs when the test comes back to
  // it; every wait for a phase or the end allows several times their length.
  const phase = { timeout: 3 * SLOW_PHASE_MS };
  const end = { timeout: 8 * SLOW_PHASE_MS };
  test.setTimeout(20 * SLOW_PHASE_MS);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");
  await newChat(page);
  const first = `${QUESTION} mock:slow-read mock:slow-write`;
  await ask(page, first);
  await expect(phaseText(page)).toHaveText(de.phase.reading_sources, phase);
  await newChat(page);
  await expect(page.getByRole("heading", { level: 2, name: de.empty.title })).toBeVisible();
  await expect(page.locator("#answer article")).toHaveCount(0);
  await expect(page.locator("#evidence h2")).toHaveText(de.sources.none);
  const nav = page.locator("nav").first();
  await expect(nav.getByRole("button", { name: first })).toBeVisible();
  await shots(page, "new-chat-while-running");
  const second = "Wo steht das Ergebnis?";
  await ask(page, second);
  await expect(phaseText(page)).toHaveText(de.phase.waiting, phase);
  await nav.getByRole("button", { name: first }).click();
  await expect(page.locator("#answer article")).toContainText(first);
  await expect(phaseText(page)).toBeVisible();
  await expect(page.getByRole("button", { name: de.cancel, exact: true })).toBeVisible();
  // Both end and are saved; evidence and title belong to the chat on display.
  await expect(page.locator("#answer article footer")).toBeVisible(end);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(first);
  await expect(page.locator("#evidence h2")).toHaveText("Quellen (2) zu Frage 1");
  await expect(page.locator("#evidence .head")).toContainText(first);
  await nav.getByRole("button", { name: second }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(second);
  await expect(page.locator("#answer article footer")).toBeVisible(end);
  await expect(page.locator("#evidence .head")).toContainText(second);
  const titles = await page.evaluate(async () => {
    const mock = (globalThis as unknown as { __ragoxideMock: import("../../api").MockBackend })
      .__ragoxideMock;
    return (await mock.api.list_chats()).map((c) => c.title);
  });
  expect(titles).toContain(first);
  expect(titles).toContain(second);
});

test("KC77 too little space: Laden locked, the dialog says how much is missing", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/?mock-scenario=download-disk-full");
  await page.evaluate(async () => {
    const shell = (
      globalThis as unknown as { __ragoxideShell: import("../../shell/setup.svelte").MockShell }
    ).__ragoxideShell;
    await shell.setup.provide(null, "fast").catch(() => {});
    await shell.setup.provide(null, "fast");
  });
  await newChat(page);
  await stepper(page).click();
  await page.getByRole("menuitemradio", { name: /Ausgewogen/ }).click();
  const dialog = page.getByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
  await expect(dialog).toContainText("Im Modellordner sind nur 3,0 GB frei.");
  await expect(dialog).toContainText(/Es fehlen \d+,\d GB\./);
  await expect(dialog.getByRole("button", { name: de.download.load, exact: true })).toBeDisabled();
  await expect(dialog.getByRole("button", { name: "Bei „Schnell“ bleiben" })).toBeFocused();
  await shots(page, "dialog-too-little-space");
});

for (const width of [960, 1100, 560]) {
  test(`KC78 a follow-up in a stored chat follows paragraph by paragraph; wheel and keys stop it (${width} px)`, async ({
    page,
  }) => {
    test.setTimeout(60_000);
    await page.setViewportSize({ width, height: 800 });
    await page.goto("/");
    await openChat(page, "Wie hoch ist das Budget?");
    // One paragraph per delta: more than 48px at once, and the earlier answer reflows
    // ("1 Quelle zeigen"), which makes the browser's scroll anchoring move the view.
    await streamWith(page, 40, 130);
    await ask(page, "stream: Folgefrage");
    const answer = page.locator("#answer article").nth(1);
    await expect(answer).toContainText("Absatz 6:");
    await atEnd(page);
    expectFollowing(await behindWhileGrowing(page, 1000));
    // Mouse wheel up: it stays.
    await scrollThread(page, -600);
    const wheeled = await threadEnd(page);
    await expect(answer).toContainText("Absatz 12:");
    let end = await threadEnd(page);
    expect(end.scrollTop).toBe(wheeled.scrollTop);
    expect(end.distance).toBeGreaterThan(48);
    // Back to the end with the wheel: it follows again.
    await scrollThread(page, "end");
    await expect(answer).toContainText("Absatz 18:");
    await atEnd(page);
    expectFollowing(await behindWhileGrowing(page, 1000));
    // Page up with the keyboard: it stays.
    await scrollWithKey(page, "PageUp");
    const keyed = await threadEnd(page);
    expect(keyed.distance).toBeGreaterThan(48);
    await expect(answer).toContainText("Absatz 26:");
    end = await threadEnd(page);
    expect(end.scrollTop).toBe(keyed.scrollTop);
    // End with the keyboard: it follows again, up to the footer after `done`.
    await scrollWithKey(page, "End");
    await expect(answer).toContainText("Absatz 32:");
    await atEnd(page);
    await expect(answer.locator("footer")).toBeVisible({ timeout: 20_000 });
    await page.waitForTimeout(200);
    await atEnd(page);
    await expect(answer.locator("footer")).toBeInViewport();
  });
}

for (const width of [960, 560]) {
  test(`KC78 keys a menu, the divider or a source row uses do not end following (${width} px)`, async ({
    page,
  }) => {
    test.setTimeout(60_000);
    await page.setViewportSize({ width, height: 800 });
    await page.goto("/");
    await openChat(page, "Wie hoch ist das Budget?");
    await streamWith(page, 200, 150);
    await ask(page, "stream: Folgefrage");
    const answer = page.locator("#answer article").nth(1);
    await expect(answer).toContainText("Absatz 4:");
    await atEnd(page);
    const stillFollowing = async (label: string, until: string) => {
      await expect(answer).toContainText(until);
      await atEnd(page, label);
      expectFollowing(await behindWhileGrowing(page, 600), label);
    };
    // The stepper's menu: arrows, Home and Escape move inside the menu, not the view.
    await stepper(page).click();
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("ArrowUp");
    await page.keyboard.press("Home");
    await page.keyboard.press("Escape");
    await stillFollowing("stepper menu", "Absatz 12:");
    // A source row in the evidence column.
    await page.evaluate(() =>
      document.querySelector<HTMLElement>("#evidence ul button")?.focus({ preventScroll: true }),
    );
    await page.keyboard.press("ArrowUp");
    await stillFollowing("source row", "Absatz 20:");
    if (width >= 700) {
      // Home on the shell's divider widens the evidence column: a reflow, no movement.
      await page.evaluate(() =>
        document.querySelector<HTMLElement>("#evidence-divider")?.focus({ preventScroll: true }),
      );
      await page.keyboard.press("Home");
      await stillFollowing("divider", "Absatz 28:");
    }
    // A key that does scroll the thread still stops it.
    await scrollWithKey(page, "PageUp");
    const stopped = await threadEnd(page);
    expect(stopped.distance).toBeGreaterThan(48);
    await expect(answer).toContainText("Absatz 40:");
    expect((await threadEnd(page)).scrollTop).toBe(stopped.scrollTop);
  });
}

for (const width of [960, 560]) {
  test(`KC78 small steps of the wheel or the arrow key add up and stop following (${width} px)`, async ({
    page,
  }) => {
    test.setTimeout(120_000);
    await page.setViewportSize({ width, height: 800 });
    await page.goto("/");
    await openChat(page, "Wie hoch ist das Budget?");
    await streamWith(page, 600, 150);
    await ask(page, "stream: Folgefrage");
    const answer = page.locator("#answer article").nth(1);
    await expect(answer).toContainText("Absatz 4:");
    // Back at the end and following: in the narrow window the page scrolls on past the
    // end of the thread (the evidence below it); following takes over once the end of the
    // thread is back at the bottom.
    const backAtEnd = async () => {
      await scrollThread(page, "end");
      await expect
        .poll(async () => Math.round((await threadEnd(page)).distance), { timeout: 10_000 })
        .toBe(0);
      await page.waitForTimeout(400);
    };
    const staysUp = async (label: string) => {
      await page.waitForTimeout(500);
      const stopped = await threadEnd(page);
      expect(stopped.distance, label).toBeGreaterThan(48);
      await page.waitForTimeout(1500);
      expect((await threadEnd(page)).scrollTop, label).toBe(stopped.scrollTop);
    };
    // Wheel steps of a few pixels, as a touchpad or a fine wheel sends them.
    for (const [by, every, times] of [
      [-8, 16, 60],
      [-20, 100, 10],
      [-40, 300, 6],
      [-20, 500, 4],
      [-40, 1000, 3],
    ]) {
      await backAtEnd();
      await scrollThread(page, by);
      for (let i = 1; i < times; i++) {
        await page.waitForTimeout(every);
        await page.mouse.wheel(0, by);
      }
      await staysUp(`${times} × ${by} px every ${every} ms`);
    }
    // One step within the threshold keeps following; a second one 2 s later adds to it.
    // The 2 s are measured, so that the first step has not lapsed (5 s) under load (KC83).
    await backAtEnd();
    const first = Date.now();
    await scrollThread(page, -30);
    await expect(answer).toContainText(`Absatz ${await paragraphs(page, 3)}:`);
    await atEnd(page, "one step of 30 px");
    await page.waitForTimeout(Math.max(0, 2000 - (Date.now() - first)));
    expect(Date.now() - first, "time between the two steps").toBeLessThan(4000);
    await scrollThread(page, -30);
    await staysUp("two steps of 30 px, 2 s apart");
    // The arrow key, 40px a press, pressed again and again.
    await backAtEnd();
    await page.evaluate(() =>
      document.querySelector<HTMLElement>("#answer .turn .marker")?.focus({ preventScroll: true }),
    );
    for (let i = 0; i < 3; i++) {
      await page.keyboard.press("ArrowUp");
      await page.waitForTimeout(1000);
    }
    await staysUp("3 × arrow up, 1 s apart");
  });
}

for (const width of [960, 560]) {
  test(`KC78 a small step is kept for 5 s, then the view goes to the end, also after done (${width} px)`, async ({
    page,
  }) => {
    test.setTimeout(60_000);
    await page.setViewportSize({ width, height: 800 });
    await page.goto("/");
    await openChat(page, "Wie hoch ist das Budget?");
    await streamWith(page, 24, 150);
    await ask(page, "stream: Folgefrage");
    const answer = page.locator("#answer article").nth(1);
    // 20 paragraphs of 150 ms: wait five times as long as the stream needs (KC83).
    await expect(answer).toContainText("Absatz 20:", { timeout: 5 * 20 * 150 });
    await expect.poll(async () => Math.round((await threadEnd(page)).distance)).toBe(0);
    // A step within the threshold shortly before the end of the answer: it is kept while
    // the rest arrives, then the view goes to the end with the footer.
    await scrollThread(page, -30);
    await expect(answer.locator("footer")).toBeVisible();
    await page.waitForTimeout(500);
    expect(Math.round((await threadEnd(page)).distance)).toBe(30);
    await page.waitForTimeout(5000);
    expect(Math.round((await threadEnd(page)).distance)).toBe(0);
    await expect(answer.locator("footer")).toBeInViewport();
  });
}

for (const width of [960, 560]) {
  for (const delay of [100, 200]) {
    test(`KC78 a growth ${delay} ms after a small wheel step is followed at once, the step kept (${width} px)`, async ({
      page,
    }) => {
      test.setTimeout(60_000);
      await page.setViewportSize({ width, height: 800 });
      await page.goto("/");
      await openChat(page, "Wie hoch ist das Budget?");
      await streamByHand(page);
      await ask(page, "stream: Folgefrage");
      const answer = page.locator("#answer article").nth(1);
      await page.waitForFunction(() => "__paragraphs" in globalThis);
      const send = () =>
        page.evaluate(() =>
          (globalThis as unknown as { __paragraphs: (n: number) => void }).__paragraphs(1),
        );
      for (let i = 0; i < 14; i++) {
        await send();
        await page.waitForTimeout(150);
      }
      await expect(answer).toContainText("Absatz 14:");
      await expect.poll(async () => Math.round((await threadEnd(page)).distance)).toBe(0);
      // A small step, then, still inside the 250 ms the user's scrolling counts as going
      // on, the last growth; nothing grows after it.
      await scrollThread(page, -30, 0);
      await page.waitForTimeout(delay);
      await send();
      await expect(answer).toContainText("Absatz 15:");
      for (const at of [800, 2000]) {
        await page.waitForTimeout(at === 800 ? 800 - delay : 1200);
        expect(Math.round((await threadEnd(page)).distance), `after ${at} ms`).toBe(30);
      }
    });
  }
}

/** Opens Edge with real scroll bars (headless Edge hides them) and the chat. */
async function withScrollBars(width: number, run: (page: Page) => Promise<void>) {
  const edge = await chromium.launch({
    channel: "msedge",
    ignoreDefaultArgs: ["--hide-scrollbars"],
  });
  try {
    const page = await edge.newPage({ viewport: { width, height: 800 } });
    await page.goto("http://localhost:5173/");
    await run(page);
  } finally {
    await edge.close();
  }
}

/** The middle of the scroll bar's thumb of the element that scrolls the thread. */
async function thumb(page: Page) {
  return page.evaluate(() => {
    let box = document.querySelector<HTMLElement>("#answer .thread");
    while (box && !["auto", "scroll"].includes(getComputedStyle(box).overflowY)) {
      box = box.parentElement;
    }
    const rect = box!.getBoundingClientRect();
    const bar = box!.offsetWidth - box!.clientWidth;
    // The track lies between the two arrow buttons, each as high as the bar is wide.
    const track = box!.clientHeight - 2 * bar;
    const size = Math.max(bar, (track * box!.clientHeight) / box!.scrollHeight);
    const share = box!.scrollTop / (box!.scrollHeight - box!.clientHeight);
    return {
      x: rect.left + box!.clientWidth + bar / 2,
      y: rect.top + bar + (track - size) * share + size / 2,
      bar,
    };
  });
}

for (const width of [960, 560]) {
  test(`KC78 the thumb held still, then dragged up, stops following (${width} px)`, async () => {
    test.setTimeout(60_000);
    await withScrollBars(width, async (page) => {
      await openChat(page, "Wie hoch ist das Budget?");
      await streamWith(page, 300, 150);
      await ask(page, "stream: Folgefrage");
      const answer = page.locator("#answer article").nth(1);
      await expect(answer).toContainText("Absatz 8:");
      await atEnd(page);
      const at = await thumb(page);
      expect(at.bar).toBeGreaterThan(0);
      await page.mouse.move(at.x, at.y);
      await page.mouse.down();
      // Held still for longer than the window of an input: it keeps following.
      await page.waitForTimeout(2500);
      await expect
        .poll(async () => (await threadEnd(page)).distance, { message: "held still" })
        .toBeLessThanOrEqual(48);
      // Then up by 300px, still pressed, and released.
      for (let i = 1; i <= 10; i++) {
        await page.mouse.move(at.x, at.y - i * 30);
        await page.waitForTimeout(50);
      }
      await page.waitForTimeout(1000);
      const dragged = await threadEnd(page);
      expect(dragged.distance, "dragged up, still pressed").toBeGreaterThan(48);
      await page.mouse.up();
      await page.waitForTimeout(500);
      const stopped = await threadEnd(page);
      expect(stopped.scrollTop, "released").toBe(dragged.scrollTop);
      await page.waitForTimeout(2000);
      expect((await threadEnd(page)).scrollTop, "2 s later").toBe(stopped.scrollTop);
    });
  });
}

/**
 * Lets the next question starting with "stream:" be answered paragraph by paragraph by
 * hand: `__paragraphs(n)` sends the next `n` paragraphs at once.
 */
async function streamByHand(page: Page) {
  await page.evaluate(() => {
    type Mock = import("../../api").MockBackend;
    const mock = (globalThis as unknown as { __ragoxideMock: Mock }).__ragoxideMock;
    const original = mock.api.ask;
    mock.api.ask = async (args, onEvent) => {
      if (!args.question.startsWith("stream:")) return original(args, onEvent);
      const hits = await mock.api.search({
        text: "Wie oft wird ein Gerät geprüft?",
        limit: 3,
        documents: null,
        mode: "hybrid",
      });
      onEvent({ type: "sources", hits });
      onEvent({ type: "status", phase: "writing", progress: null });
      let count = 0;
      (globalThis as unknown as { __paragraphs: (n: number) => void }).__paragraphs = (n) => {
        for (let i = 0; i < n; i++) {
          count += 1;
          onEvent({
            type: "delta",
            text: `Absatz ${count}: Jedes Gerät wird mindestens einmal jährlich geprüft, und das Ergebnis steht im Prüfprotokoll [1].\n\n`,
          });
        }
      };
      // Never done: the test ends with the page.
      return new Promise(() => {});
    };
  });
}

for (const width of [960, 1440]) {
  test(`KC78 back at the end with the wheel while the thread grows before the wheel event (${width} px)`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 800 });
    await page.goto("/");
    await openChat(page, "Wie hoch ist das Budget?");
    await streamByHand(page);
    await ask(page, "stream: Folgefrage");
    const paragraphs = (n: number) =>
      page.evaluate(
        (n) => (globalThis as unknown as { __paragraphs: (n: number) => void }).__paragraphs(n),
        n,
      );
    const answer = page.locator("#answer article").nth(1);
    await page.waitForFunction(() => "__paragraphs" in globalThis);
    await paragraphs(30);
    await expect(answer).toContainText("Absatz 30:");
    await expect.poll(async () => Math.round((await threadEnd(page)).distance)).toBe(0);
    // Up with the wheel, and the window of that input runs out.
    await scrollThread(page, -600);
    await page.waitForTimeout(1700);
    await paragraphs(3);
    await expect(answer).toContainText("Absatz 33:");
    expect((await threadEnd(page)).distance).toBeGreaterThan(48);
    // The order Edge produces under load: the wheel scrolls the view to the end, the
    // thread grows, the scroll event and the ResizeObserver run, and only then the wheel
    // event arrives (its time stamp is from before the growth).
    await page.evaluate(async () => {
      let box = document.querySelector<HTMLElement>("#answer .thread");
      while (box && !["auto", "scroll"].includes(getComputedStyle(box).overflowY)) {
        box = box.parentElement;
      }
      const wheel = new WheelEvent("wheel", { deltaY: 100_000, bubbles: true });
      box!.scrollTop = box!.scrollHeight;
      (globalThis as unknown as { __paragraphs: (n: number) => void }).__paragraphs(3);
      await new Promise((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(resolve, 50))),
      );
      box!.dispatchEvent(wheel);
    });
    await expect(answer).toContainText("Absatz 36:");
    // It follows again: each further growth stays at the end.
    for (let i = 0; i < 4; i++) {
      await page.waitForTimeout(300);
      await paragraphs(2);
      await page.waitForTimeout(300);
      await atEnd(page, `growth ${i + 1}`);
    }
  });
}

/** The number of the paragraph `ahead` paragraphs after the last one shown so far. */
async function paragraphs(page: Page, ahead: number) {
  const text = (await page.locator("#answer article").nth(1).textContent()) ?? "";
  const shown = [...text.matchAll(/Absatz (\d+):/g)].map((m) => Number(m[1]));
  return Math.max(0, ...shown) + ahead;
}

test("KC78 a very short click on the scroll bar track stops following", async () => {
  test.setTimeout(60_000);
  // Headless Edge hides scroll bars (overlay); this context gets real ones.
  const edge = await chromium.launch({
    channel: "msedge",
    ignoreDefaultArgs: ["--hide-scrollbars"],
  });
  const page = await edge.newPage({ viewport: { width: 960, height: 800 } });
  try {
    await page.goto("http://localhost:5173/");
    await openChat(page, "Wie hoch ist das Budget?");
    await streamWith(page, 200, 150);
    await ask(page, "stream: Folgefrage");
    const answer = page.locator("#answer article").nth(1);
    await expect(answer).toContainText("Absatz 8:", { timeout: 5 * 8 * 150 });
    await atEnd(page);
    const thread = (await page.locator("#answer .thread").boundingBox())!;
    const bar = await page.evaluate(() => {
      const t = document.querySelector<HTMLElement>("#answer .thread")!;
      return t.offsetWidth - t.clientWidth;
    });
    expect(bar).toBeGreaterThan(0);
    // Down and up at once (< 30 ms) on the track above the thumb: one page up.
    await page.mouse.move(thread.x + thread.width - bar / 2, thread.y + 30);
    await page.mouse.down();
    await page.mouse.up();
    await page.waitForTimeout(600);
    const stopped = await threadEnd(page);
    expect(stopped.distance).toBeGreaterThan(48);
    // Twelve more paragraphs of 150 ms: five times as long as the stream needs (KC83).
    await expect(answer).toContainText("Absatz 20:", { timeout: 5 * 12 * 150 });
    expect((await threadEnd(page)).scrollTop).toBe(stopped.scrollTop);
  } finally {
    await edge.close();
  }
});
