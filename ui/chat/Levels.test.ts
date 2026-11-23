// The levels of the answer model (ADR 0060, 0064 to 0070, 0126): stepper and help line,
// the menu with wait and download state, the download dialog, pausing and resuming,
// a failed download, a refused question, "Erneut fragen" and the way out after 30 s.
import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { AnswerLevel, MockBackend } from "$api";
import { answerLevel, i18n, setup, t, workspace } from "$shell";
import { startStatus } from "$shell/status.svelte";
import { gigabytes } from "./answer";
import Chat from "./Chat.svelte";
import { answerOf, fresh, QUESTION, scripted, settle } from "./testkit.svelte";

let stops: (() => void)[] = [];

afterEach(() => {
  vi.useRealTimers();
  stops.forEach((stop) => stop());
  stops = [];
});

let current: MockBackend;

async function start(options: Parameters<typeof fresh>[0] = {}, withStatus = false) {
  current = await fresh(options);
  // The shell's progress of a download comes from `model_progress` (status bar).
  if (withStatus) stops.push(await startStatus());
  render(Chat);
  await settle();
  return current;
}

const stepper = () => screen.getByRole("button", { name: /^(Stufe|Level): / });
const help = () => document.getElementById("question-help")!;
const field = () => screen.getByLabelText("Frage an deine Dokumente") as HTMLTextAreaElement;

async function openMenu() {
  await fireEvent.click(stepper());
  await settle(1);
  return screen.getByRole("menu", { name: "Stufe wählen" });
}

async function choose(name: RegExp) {
  const menu = await openMenu();
  await fireEvent.click(within(menu).getByRole("menuitemradio", { name }));
  await settle();
}

async function openChat(id: string) {
  workspace.openChat(id);
  await waitFor(() => expect(screen.getAllByRole("article").length).toBeGreaterThan(0));
  await settle();
}

describe("stepper and menu", () => {
  test("KC42 the stepper shows the chosen level, the help line its price; Gründlich warns", async () => {
    await start({ answerLevel: "balanced", scenarios: ["all-levels"] });
    expect(stepper().textContent).toBe("Ausgewogen");
    expect(help().textContent?.trim()).toBe(
      "Ausgewogen: Antwort beginnt meist nach etwa einer Minute.",
    );
    expect(help().querySelector("svg")).toBeNull();
    await choose(/Gründlich/);
    expect(stepper().textContent).toBe("Gründlich");
    expect(help().classList.contains("warning")).toBe(true);
    expect(help().querySelector("svg")).toBeTruthy();
    expect(help().textContent).toContain(
      "Jede Frage dauert mehrere Minuten, bis du die Stufe änderst.",
    );
  });

  test("KC43 the menu names wait, state and that the level stays; KC61 arrows and Escape", async () => {
    await start();
    const menu = await openMenu();
    const items = within(menu).getAllByRole("menuitemradio");
    expect(items.map((i) => i.querySelector(".name")?.textContent)).toEqual([
      "Schnell",
      "Ausgewogen",
      "Gründlich",
    ]);
    expect(items[0].textContent).toContain("beginnt meist innerhalb einer halben Minute");
    expect(items[1].textContent).toContain("beginnt meist nach etwa einer Minute");
    expect(items[2].textContent).toContain(
      "beginnt erst nach mehreren Minuten und entsteht langsamer, als du liest",
    );
    expect(items[0].textContent).toContain("Bereit");
    const balanced = setup.level("balanced")!;
    const size = gigabytes(balanced.download_bytes, "de");
    expect(items[1].textContent).toMatch(
      new RegExp(
        `Noch nicht geladen, (rund|mindestens) ${size} GB\\. Wird beim Wählen angeboten\\.`,
      ),
    );
    expect(menu.textContent).toContain("Gilt für alle weiteren Fragen, bis du sie änderst.");
    expect(items[0].getAttribute("aria-checked")).toBe("true");
    // Keyboard: the chosen level has the focus; arrows move; Escape returns to the button.
    expect(document.activeElement).toBe(items[0]);
    await fireEvent.keyDown(menu, { key: "ArrowDown" });
    expect(document.activeElement).toBe(items[1]);
    await fireEvent.keyDown(menu, { key: "End" });
    expect(document.activeElement).toBe(items[2]);
    await fireEvent.keyDown(menu, { key: "ArrowDown" });
    expect(document.activeElement).toBe(items[0]);
    await fireEvent.keyDown(menu, { key: "Escape" });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(stepper());
  });

  test("KC44 choosing a ready level changes it for all further questions", async () => {
    const mock = await start({ scenarios: ["all-levels"] });
    const { asked } = scripted(mock);
    await choose(/Ausgewogen/);
    expect(answerLevel.current).toBe("balanced");
    expect((await mock.api.get_settings()).answer_level).toBe("balanced");
    expect(stepper().textContent).toBe("Ausgewogen");
    expect(help().textContent).toContain("Ausgewogen:");
    await fireEvent.input(field(), { target: { value: QUESTION } });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    expect(asked[0].args.level).toBe("balanced");
  });

  test("KC45 KC79c below the minimum the higher levels are visible, not choosable, with the memory", async () => {
    const mock = await start({ scenarios: ["weak-machine"] });
    await setup.provide(null, "fast");
    await settle();
    // The price of Schnell stays; the note comes on top (KC79 c).
    expect(help().textContent?.trim()).toBe(
      "Schnell: Antwort beginnt meist innerhalb einer halben Minute.",
    );
    const weak = document.getElementById("question-weak")!;
    expect(weak.textContent).toContain("Auf diesem Rechner gibt es nur „Schnell“");
    expect(field().getAttribute("aria-describedby")).toContain("question-weak");
    const menu = await openMenu();
    const [, balanced, thorough] = within(menu).getAllByRole("menuitemradio");
    for (const item of [balanced, thorough]) {
      expect(item.getAttribute("aria-disabled")).toBe("true");
      expect(item.textContent).toContain("braucht mindestens 16 GB Arbeitsspeicher");
      expect(item.textContent).toContain("dieser Rechner hat 4 GB");
    }
    await fireEvent.click(balanced);
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(answerLevel.current).toBe("fast");
    await fireEvent.keyDown(menu, { key: "Escape" });
    // The same in "Erneut fragen".
    await openChat("chat-example");
    await fireEvent.click(screen.getByRole("button", { name: /Erneut fragen/ }));
    const again = screen.getByRole("menu", { name: "Erneut fragen in Stufe" });
    const item = within(again).getByRole("menuitem", { name: /In „Gründlich“/ });
    expect(item.getAttribute("aria-disabled")).toBe("true");
    expect(item.textContent).toContain("dieser Rechner hat 4 GB");
    const askAgain = vi.spyOn(mock.api, "ask_again");
    await fireEvent.click(item);
    expect(askAgain).not.toHaveBeenCalled();
  });

  test("KC45 a stored level the machine does not offer appears as Schnell", async () => {
    await start({ scenarios: ["weak-machine"], answerLevel: "thorough" });
    expect(stepper().textContent).toBe("Schnell");
    expect(answerLevel.current).toBe("fast");
  });
});

describe("download of a level", () => {
  test("KC46 KC62 the dialog asks first, names size and space, starts on the safe choice", async () => {
    const mock = await start();
    const provide = vi.spyOn(mock.api, "provide_models");
    await fireEvent.input(field(), { target: { value: "Meine Frage" } });
    const button = stepper();
    await choose(/Ausgewogen/);
    const dialog = screen.getByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
    const size = gigabytes(setup.level("balanced")!.download_bytes, "de");
    expect(dialog.textContent).toMatch(new RegExp(`(rund|mindestens) ${size} GB aus dem Internet`));
    expect(dialog.textContent).toContain("Im Modellordner sind 120,0 GB frei");
    expect(dialog.textContent).toContain("läuft im Hintergrund");
    expect(dialog.textContent).toContain("Bis es fertig ist, antwortet „Schnell“.");
    expect(dialog.textContent).toContain("Deine Frage bleibt im Eingabefeld.");
    const stay = within(dialog).getByRole("button", { name: "Bei „Schnell“ bleiben" });
    expect(within(dialog).getByRole("button", { name: "Laden" })).toBeTruthy();
    await settle(1);
    expect(document.activeElement).toBe(stay);
    // Escape is the safe choice; the focus goes back to the stepper.
    await fireEvent.keyDown(dialog, { key: "Escape" });
    await settle(1);
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(button);
    await choose(/Ausgewogen/);
    await fireEvent.click(screen.getByRole("button", { name: "Bei „Schnell“ bleiben" }));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(provide).not.toHaveBeenCalled();
    expect(field().value).toBe("Meine Frage");
    expect(answerLevel.current).toBe("fast");
  });

  test("KC46 KC77 too little space for a known size: Laden is locked and the dialog says how much is missing", async () => {
    const mock = await start({ scenarios: ["download-disk-full"] });
    await setup.provide(null, "fast").catch(() => {});
    await setup.provide(null, "fast");
    await settle();
    const provide = vi.spyOn(mock.api, "provide_models");
    await choose(/Ausgewogen/);
    const dialog = screen.getByRole("dialog");
    const balanced = setup.level("balanced")!;
    expect(balanced.unknown_sizes).toBe(0);
    const lacking = gigabytes(balanced.download_bytes - 3_000_000_000, "de");
    expect(dialog.textContent).toContain("Im Modellordner sind nur 3,0 GB frei.");
    expect(dialog.textContent).toContain(`Es fehlen ${lacking} GB.`);
    const load = within(dialog).getByRole("button", { name: "Laden" }) as HTMLButtonElement;
    expect(load.disabled).toBe(true);
    await settle(1);
    expect(document.activeElement).toBe(
      within(dialog).getByRole("button", { name: "Bei „Schnell“ bleiben" }),
    );
    await fireEvent.click(load);
    expect(provide).not.toHaveBeenCalled();
    // The same in the dialog of "Erneut fragen".
    await fireEvent.keyDown(dialog, { key: "Escape" });
    await openChat("chat-example");
    await fireEvent.click(screen.getByRole("button", { name: /Erneut fragen/ }));
    await fireEvent.click(screen.getByRole("menuitem", { name: /In „Ausgewogen“/ }));
    const again = screen.getByRole("dialog");
    expect(again.textContent).toContain(`Es fehlen ${lacking} GB.`);
    expect(
      (within(again).getByRole("button", { name: "Laden" }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  test("KC77 with files of unknown size it only warns, and Laden stays possible", async () => {
    const mock = await start();
    const real = mock.api.setup_state;
    const provide = vi.fn(async () => null);
    scripted(mock, {
      setup_state: async () => {
        const state = await real();
        return {
          ...state,
          storage: { ...state.storage, free_bytes: 1_000_000_000 },
          levels: state.levels.map((l) =>
            l.level === "balanced" ? { ...l, unknown_sizes: 2 } : l,
          ),
        };
      },
      provide_models: provide,
    });
    await setup.refresh();
    await settle();
    await choose(/Ausgewogen/);
    const dialog = screen.getByRole("dialog");
    expect(dialog.textContent).toContain("mindestens");
    expect(dialog.textContent).toContain("Im Modellordner sind nur 1,0 GB frei, zu wenig für");
    expect(dialog.textContent).not.toContain("Es fehlen");
    const load = within(dialog).getByRole("button", { name: "Laden" }) as HTMLButtonElement;
    expect(load.disabled).toBe(false);
    await fireEvent.click(load);
    await waitFor(() => expect(provide).toHaveBeenCalled());
  });

  test("KC47 KC48 loading: the stepper stays, menu and help line show the percent; pause and resume in the menu", async () => {
    const mock = await start({ tickMs: 2 }, true);
    const provide = vi.spyOn(mock.api, "provide_models");
    let held = false;
    stops.push(
      await mock.listen("model_progress", () => {
        if (held || (setup.level("balanced")?.progress?.fraction ?? 0) < 0.2) return;
        held = true;
        mock.modelFiles.hold();
      }),
    );
    await choose(/Ausgewogen/);
    await fireEvent.click(screen.getByRole("button", { name: "Laden" }));
    expect(provide).toHaveBeenCalledWith(
      expect.objectContaining({ level: "balanced", source: null }),
    );
    await waitFor(() => expect(held).toBe(true), { timeout: 10_000 });
    await settle();
    expect(stepper().textContent).toBe("Schnell");
    expect(help().textContent).toMatch(
      /„Ausgewogen“ wird geladen: \d+ %\. Bis dahin antwortet „Schnell“\./,
    );
    expect(within(help()).queryByRole("button")).toBeNull();
    let menu = await openMenu();
    expect(within(menu).getByRole("menuitemradio", { name: /Ausgewogen/ }).textContent).toMatch(
      /Wird geladen: \d+ %/,
    );
    const reached = setup.level("balanced")!.progress!.fraction;
    await fireEvent.click(within(menu).getByRole("menuitem", { name: "Laden anhalten" }));
    await waitFor(() => expect(setup.level("balanced")?.downloading).toBe(false));
    mock.modelFiles.release();
    await settle();
    expect(help().textContent).toMatch(/Laden von „Ausgewogen“ angehalten bei \d+ %/);
    menu = await openMenu();
    expect(within(menu).getByRole("menuitemradio", { name: /Ausgewogen/ }).textContent).toMatch(
      /Angehalten bei \d+ %/,
    );
    // Resume continues the part already there, not from 0.
    let first: number | null = null;
    stops.push(
      await mock.listen("model_progress", () => {
        first ??= setup.level("balanced")?.progress?.fraction ?? null;
      }),
    );
    await fireEvent.click(within(menu).getByRole("menuitem", { name: "Fortsetzen" }));
    expect(provide).toHaveBeenCalledTimes(2);
    await waitFor(() => expect(first).not.toBeNull(), { timeout: 10_000 });
    expect(first!).toBeGreaterThan(reached / 2);
    // Ready: the level switches.
    await waitFor(() => expect(answerLevel.current).toBe("balanced"), { timeout: 20_000 });
    expect(stepper().textContent).toBe("Ausgewogen");
  }, 40_000);

  test("KC49 a failed download: red help line with cause, parts stay, offline package; retry by choosing", async () => {
    await start({ scenarios: ["level-download-error"] }, true);
    await choose(/Ausgewogen/);
    await fireEvent.click(screen.getByRole("button", { name: "Laden" }));
    await waitFor(() => expect(setup.level("balanced")?.error?.kind).toBe("network"));
    await settle();
    expect(help().classList.contains("danger")).toBe(true);
    expect(help().textContent).toContain(
      "Laden von „Ausgewogen“ fehlgeschlagen: keine Verbindung.",
    );
    expect(help().textContent).toContain(
      "Die geladenen Teile bleiben, ein neuer Versuch setzt dort fort.",
    );
    expect(help().textContent).toContain("Offline-Paket in den Einstellungen");
    const menu = await openMenu();
    const item = within(menu).getByRole("menuitemradio", { name: /Ausgewogen/ });
    expect(item.textContent).toContain("Wählen, um es erneut zu versuchen.");
    await fireEvent.click(item);
    expect(screen.queryByRole("dialog")).toBeNull();
    await waitFor(() => expect(answerLevel.current).toBe("balanced"), { timeout: 10_000 });
  }, 20_000);

  test("KC50 refused with model_missing: nothing asked or saved, the download is offered, the question stays", async () => {
    const mock = await start({ scenarios: ["all-levels"], answerLevel: "balanced" });
    const fast = new Set(mock.modelFiles.ofLevel("fast"));
    const generator = mock.modelFiles.ofLevel("balanced").find((key) => !fast.has(key))!;
    mock.modelFiles.remove(generator);
    const chats = (await mock.api.list_chats()).length;
    await fireEvent.input(field(), { target: { value: QUESTION } });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    const dialog = await screen.findByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
    expect(dialog.textContent).toContain("Bis es fertig ist, antwortet „Schnell“.");
    expect(field().value).toBe(QUESTION);
    expect(screen.queryAllByRole("article")).toHaveLength(0);
    expect(await mock.api.list_chats()).toHaveLength(chats);
  });

  test("KC79b Escape in the dialog after model_missing changes nothing; Bei Schnell bleiben switches to Schnell", async () => {
    const mock = await start({ scenarios: ["all-levels"], answerLevel: "balanced" });
    const fast = new Set(mock.modelFiles.ofLevel("fast"));
    mock.modelFiles.remove(mock.modelFiles.ofLevel("balanced").find((key) => !fast.has(key))!);
    await fireEvent.input(field(), { target: { value: QUESTION } });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    let dialog = await screen.findByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
    await fireEvent.keyDown(dialog, { key: "Escape" });
    await settle();
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(answerLevel.current).toBe("balanced");
    expect((await mock.api.get_settings()).answer_level).toBe("balanced");
    expect(field().value).toBe(QUESTION);
    // The next send offers it again.
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    dialog = await screen.findByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
    await fireEvent.click(within(dialog).getByRole("button", { name: "Bei „Schnell“ bleiben" }));
    await waitFor(() => expect(answerLevel.current).toBe("fast"));
    expect((await mock.api.get_settings()).answer_level).toBe("fast");
    expect(field().value).toBe(QUESTION);
  });
});

/** Removes a model file only `level` needs; the other levels stay ready. */
function removeOnly(mock: MockBackend, level: AnswerLevel) {
  const others = new Set(
    (["fast", "balanced", "thorough"] as const)
      .filter((other) => other !== level)
      .flatMap((other) => mock.modelFiles.ofLevel(other)),
  );
  mock.modelFiles.remove(mock.modelFiles.ofLevel(level).find((key) => !others.has(key))!);
}

/**
 * The three ways into the download dialog (KC46, KC50, KC51), each with a stored level
 * other than Schnell, so that a switch to Schnell shows in the stored value.
 */
const entries: {
  name: string;
  /** The level the dialog offers. */
  level: AnswerLevel;
  /** The stored level before the dialog. */
  before: AnswerLevel;
  /** The second button "Bei „…“ bleiben" names this level and stores it afterwards. */
  stay: AnswerLevel;
  /** The level in effect while the offered level loads after "Laden" (KC81). */
  loading: AnswerLevel;
  open: () => Promise<MockBackend>;
}[] = [
  {
    name: "chosen in the menu (KC46)",
    level: "balanced",
    before: "thorough",
    stay: "thorough",
    loading: "thorough",
    async open() {
      const mock = await start({ scenarios: ["all-levels"], answerLevel: "thorough" }, true);
      removeOnly(mock, "balanced");
      await setup.refresh();
      await settle();
      await fireEvent.input(field(), { target: { value: QUESTION } });
      await choose(/Ausgewogen/);
      return mock;
    },
  },
  {
    name: "after model_missing (KC50)",
    level: "balanced",
    before: "balanced",
    stay: "fast",
    loading: "fast",
    async open() {
      const mock = await start({ scenarios: ["all-levels"], answerLevel: "balanced" }, true);
      removeOnly(mock, "balanced");
      await fireEvent.input(field(), { target: { value: QUESTION } });
      await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
      await screen.findByRole("dialog");
      return mock;
    },
  },
  {
    name: "from Erneut fragen (KC51)",
    level: "thorough",
    before: "balanced",
    stay: "balanced",
    loading: "balanced",
    async open() {
      const mock = await start({ scenarios: ["all-levels"], answerLevel: "balanced" }, true);
      removeOnly(mock, "thorough");
      await setup.refresh();
      await openChat("chat-example");
      await fireEvent.input(field(), { target: { value: QUESTION } });
      await fireEvent.click(screen.getByRole("button", { name: /Erneut fragen/ }));
      await fireEvent.click(screen.getByRole("menuitem", { name: /In „Gründlich“/ }));
      return mock;
    },
  },
];

const stored = async (mock: MockBackend) => (await mock.api.get_settings()).answer_level;
const levelName = (level: AnswerLevel) => t(`chat.level.${level}`);
const price: Record<AnswerLevel, string> = {
  fast: "Schnell: Antwort beginnt meist innerhalb einer halben Minute.",
  balanced: "Ausgewogen: Antwort beginnt meist nach etwa einer Minute.",
  thorough: "Gründlich: Jede Frage dauert mehrere Minuten, bis du die Stufe änderst.",
};

describe("KC80 only Bei Schnell bleiben after model_missing changes the level (ADR 0146, 0152)", () => {
  for (const entry of entries) {
    const stayName = `Bei „${levelName(entry.stay)}“ bleiben`;

    test(`KC80 ${entry.name}: the second button is "${stayName}" and leaves ${entry.stay} stored, the question stays`, async () => {
      const mock = await entry.open();
      const dialog = screen.getByRole("dialog", {
        name: `„${levelName(entry.level)}“ zuerst laden?`,
      });
      const provide = vi.spyOn(mock.api, "provide_models");
      const write = vi.spyOn(mock.api, "set_answer_level");
      expect(
        within(dialog)
          .getAllByRole("button")
          .map((b) => b.textContent),
      ).toEqual(["Laden", stayName]);
      expect(await stored(mock)).toBe(entry.before);
      await fireEvent.click(within(dialog).getByRole("button", { name: stayName }));
      await settle();
      expect(screen.queryByRole("dialog")).toBeNull();
      expect(await stored(mock)).toBe(entry.stay);
      expect(answerLevel.current).toBe(entry.stay);
      if (entry.stay === entry.before) expect(write).not.toHaveBeenCalled();
      else expect(write).toHaveBeenCalledWith(expect.objectContaining({ level: "fast" }));
      expect(stepper().textContent).toBe(levelName(entry.stay));
      expect(help().textContent?.trim()).toBe(price[entry.stay]);
      expect(field().value).toBe(QUESTION);
      expect(provide).not.toHaveBeenCalled();
    });

    test(`KC80 ${entry.name}: Laden provides the level and changes no level`, async () => {
      const mock = await entry.open();
      const dialog = screen.getByRole("dialog", {
        name: `„${levelName(entry.level)}“ zuerst laden?`,
      });
      const provide = vi.spyOn(mock.api, "provide_models");
      const write = vi.spyOn(mock.api, "set_answer_level");
      mock.modelFiles.hold();
      expect(await stored(mock)).toBe(entry.before);
      await fireEvent.click(within(dialog).getByRole("button", { name: "Laden" }));
      await settle();
      expect(screen.queryByRole("dialog")).toBeNull();
      expect(provide).toHaveBeenCalledWith(
        expect.objectContaining({ level: entry.level, source: null }),
      );
      expect(write).not.toHaveBeenCalled();
      expect(await stored(mock)).toBe(entry.before);
      expect(answerLevel.current).toBe(entry.before);
      expect(stepper().textContent).toBe(levelName(entry.loading));
      expect(field().value).toBe(QUESTION);
      // Ready: only a level chosen in the menu switches (KC47); the others stay.
      mock.modelFiles.release();
      await waitFor(() => expect(setup.level(entry.level)?.ready).toBe(true), {
        timeout: 10_000,
      });
      await settle();
      if (entry.level === "balanced" && entry.before === "thorough") {
        await waitFor(() => expect(answerLevel.current).toBe("balanced"));
        expect(await stored(mock)).toBe("balanced");
      } else {
        expect(write).not.toHaveBeenCalled();
        expect(await stored(mock)).toBe(entry.before);
      }
      expect(field().value).toBe(QUESTION);
    }, 20_000);

    test(`KC80 KC79b ${entry.name}: Escape changes nothing`, async () => {
      const mock = await entry.open();
      const dialog = screen.getByRole("dialog", {
        name: `„${levelName(entry.level)}“ zuerst laden?`,
      });
      const provide = vi.spyOn(mock.api, "provide_models");
      const write = vi.spyOn(mock.api, "set_answer_level");
      await fireEvent.keyDown(dialog, { key: "Escape" });
      await settle();
      expect(screen.queryByRole("dialog")).toBeNull();
      expect(write).not.toHaveBeenCalled();
      expect(provide).not.toHaveBeenCalled();
      expect(await stored(mock)).toBe(entry.before);
      expect(field().value).toBe(QUESTION);
    });
  }
});

describe("KC81 while the stored level loads, Schnell answers (ADR 0151)", () => {
  /** Stored Ausgewogen without its model, refused, "Laden" in the dialog, download held. */
  async function loading() {
    const mock = await entries[1].open();
    mock.modelFiles.hold();
    await fireEvent.click(screen.getByRole("button", { name: "Laden" }));
    await waitFor(() => expect(setup.level("balanced")?.downloading).toBe(true));
    await settle();
    return mock;
  }

  async function send() {
    await fireEvent.input(field(), { target: { value: QUESTION } });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    await settle();
  }

  test("KC81 sending during the download asks in Schnell without the dialog; the stored level stays", async () => {
    const mock = await loading();
    const ask = vi.spyOn(mock.api, "ask");
    const write = vi.spyOn(mock.api, "set_answer_level");
    await send();
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(ask).toHaveBeenCalledTimes(1);
    expect(ask.mock.calls[0][0]).toEqual(expect.objectContaining({ level: "fast" }));
    const footer = () => screen.getAllByRole("article")[0].querySelector("footer");
    await waitFor(() => expect(footer()).toBeTruthy(), { timeout: 5_000 });
    expect(footer()!.textContent).toContain("Schnell");
    expect(write).not.toHaveBeenCalled();
    expect(await stored(mock)).toBe("balanced");
    expect(answerLevel.current).toBe("balanced");
    mock.modelFiles.release();
  });

  test("KC81 stepper and help line show Schnell and the progress; no text says the loading level answers", async () => {
    const mock = await loading();
    expect(stepper().textContent).toBe("Schnell");
    expect(help().textContent).toMatch(
      /„Ausgewogen“ wird geladen: \d+ %\. Bis dahin antwortet „Schnell“\./,
    );
    expect(document.body.textContent).not.toContain("antwortet „Ausgewogen“");
    const menu = await openMenu();
    expect(
      within(menu)
        .getByRole("menuitemradio", { name: /Schnell/ })
        .getAttribute("aria-checked"),
    ).toBe("true");
    mock.modelFiles.release();
  });

  test("KC81 once ready, the stored level answers without another step", async () => {
    const mock = await loading();
    const write = vi.spyOn(mock.api, "set_answer_level");
    mock.modelFiles.release();
    await waitFor(() => expect(setup.level("balanced")?.ready).toBe(true), { timeout: 10_000 });
    await settle();
    expect(stepper().textContent).toBe("Ausgewogen");
    expect(help().textContent?.trim()).toBe(price.balanced);
    const ask = vi.spyOn(mock.api, "ask");
    await send();
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(ask).toHaveBeenCalledTimes(1);
    expect(ask.mock.calls[0][0]).toEqual(expect.objectContaining({ level: "balanced" }));
    expect(write).not.toHaveBeenCalled();
    expect(await stored(mock)).toBe("balanced");
  }, 20_000);

  test("KC81 KC48 a paused download: sending offers the download again", async () => {
    const mock = await loading();
    const menu = await openMenu();
    await fireEvent.click(within(menu).getByRole("menuitem", { name: "Laden anhalten" }));
    await waitFor(() => expect(setup.level("balanced")?.downloading).toBe(false));
    mock.modelFiles.release();
    await settle();
    expect(stepper().textContent).toBe("Ausgewogen");
    const ask = vi.spyOn(mock.api, "ask");
    await send();
    expect(ask).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" })).toBeTruthy();
    expect(field().value).toBe(QUESTION);
    expect(await stored(mock)).toBe("balanced");
  });

  test("KC81 KC49 a failed download: sending offers the download again; loading again from the menu, Schnell answers", async () => {
    const mock = await entries[1].open();
    const missing = mock.modelFiles.ofLevel("balanced").find((key) => !mock.modelFiles.ready(key))!;
    mock.modelFiles.failChecksumOnce(missing);
    await fireEvent.click(screen.getByRole("button", { name: "Laden" }));
    await waitFor(() => expect(setup.level("balanced")?.error).toBeTruthy(), { timeout: 10_000 });
    await settle();
    expect(stepper().textContent).toBe("Ausgewogen");
    const ask = vi.spyOn(mock.api, "ask");
    await send();
    expect(ask).not.toHaveBeenCalled();
    const dialog = screen.getByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
    await fireEvent.keyDown(dialog, { key: "Escape" });
    await settle();
    // The stored level loads again, started from the menu: Schnell answers meanwhile.
    mock.modelFiles.hold();
    const menu = await openMenu();
    await fireEvent.click(within(menu).getByRole("menuitemradio", { name: /Ausgewogen/ }));
    await waitFor(() => expect(setup.level("balanced")?.downloading).toBe(true));
    await settle();
    expect(stepper().textContent).toBe("Schnell");
    await send();
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(ask).toHaveBeenCalledTimes(1);
    expect(ask.mock.calls[0][0]).toEqual(expect.objectContaining({ level: "fast" }));
    expect(await stored(mock)).toBe("balanced");
    mock.modelFiles.release();
  }, 20_000);

  test("KC81 KC49 the stored level failed: the help line keeps cause, parts and offline package, and claims no level answers", async () => {
    const mock = await entries[1].open();
    const missing = mock.modelFiles.ofLevel("balanced").find((key) => !mock.modelFiles.ready(key))!;
    mock.modelFiles.failChecksumOnce(missing);
    await fireEvent.click(screen.getByRole("button", { name: "Laden" }));
    await waitFor(() => expect(setup.level("balanced")?.error).toBeTruthy(), { timeout: 10_000 });
    await settle();
    expect(help().classList.contains("danger")).toBe(true);
    expect(help().textContent).toContain("Laden von „Ausgewogen“ fehlgeschlagen:");
    expect(help().textContent).toContain(
      "Die geladenen Teile bleiben, ein neuer Versuch setzt dort fort.",
    );
    expect(help().textContent).toContain("Offline-Paket in den Einstellungen");
    expect(help().textContent).not.toContain("antwortet");
    const menu = await openMenu();
    expect(within(menu).getByRole("menuitemradio", { name: /Ausgewogen/ }).textContent).toContain(
      "Wählen, um es erneut zu versuchen.",
    );
    expect(document.body.textContent).not.toMatch(/antwortet „Ausgewogen“|Bis dahin antwortet/);
  }, 20_000);

  test("KC81 KC48 the stored level paused: the help line names the pause and claims no level answers", async () => {
    const mock = await loading();
    const menu = await openMenu();
    await fireEvent.click(within(menu).getByRole("menuitem", { name: "Laden anhalten" }));
    await waitFor(() => expect(setup.level("balanced")?.downloading).toBe(false));
    mock.modelFiles.release();
    await settle();
    expect(help().textContent).toMatch(/Laden von „Ausgewogen“ angehalten bei \d+ %/);
    expect(help().textContent).not.toContain("antwortet");
    expect(document.body.textContent).not.toMatch(/antwortet „Ausgewogen“|Bis dahin antwortet/);
  });
});

describe("KC49 KC82 another level failed, the stored one has its model", () => {
  test("KC49 KC82 the red help line names the stored level as answering", async () => {
    const mock = await start({ scenarios: ["all-levels"], answerLevel: "thorough" }, true);
    removeOnly(mock, "balanced");
    await setup.refresh();
    await settle();
    const ready = new Set([
      ...mock.modelFiles.ofLevel("fast"),
      ...mock.modelFiles.ofLevel("thorough"),
    ]);
    mock.modelFiles.failChecksumOnce(
      mock.modelFiles.ofLevel("balanced").find((key) => !ready.has(key))!,
    );
    await choose(/Ausgewogen/);
    await fireEvent.click(screen.getByRole("button", { name: "Laden" }));
    await waitFor(() => expect(setup.level("balanced")?.error).toBeTruthy(), { timeout: 10_000 });
    await settle();
    expect(help().classList.contains("danger")).toBe(true);
    expect(help().textContent).toContain("Laden von „Ausgewogen“ fehlgeschlagen:");
    expect(help().textContent).toContain("Bis dahin antwortet „Gründlich“.");
    expect(stepper().textContent).toBe("Gründlich");
  }, 20_000);
});

describe("KC82 no text names a level without its model as answering", () => {
  const claim =
    /antwortet „Ausgewogen“|Bis dahin antwortet|Bis es fertig ist, antwortet „Ausgewogen“/;

  /** Stored Ausgewogen without its model, the dialog after sending closed with Escape. */
  async function storedWithoutModel() {
    const mock = await start({ answerLevel: "balanced" }, true);
    await fireEvent.input(field(), { target: { value: QUESTION } });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    const dialog = await screen.findByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
    await fireEvent.keyDown(dialog, { key: "Escape" });
    await settle();
    expect(await stored(mock)).toBe("balanced");
    expect(setup.level("balanced")?.ready).toBe(false);
    return mock;
  }

  test("KC82 loading a third level from the menu: neither dialog nor help line names Ausgewogen as answering", async () => {
    const mock = await storedWithoutModel();
    await choose(/Gründlich/);
    const dialog = screen.getByRole("dialog", { name: "„Gründlich“ zuerst laden?" });
    expect(dialog.textContent).toContain("läuft im Hintergrund");
    expect(dialog.textContent).not.toContain("antwortet");
    // The second button keeps the level as it was (KC80).
    expect(within(dialog).getByRole("button", { name: "Bei „Ausgewogen“ bleiben" })).toBeTruthy();
    mock.modelFiles.hold();
    await fireEvent.click(within(dialog).getByRole("button", { name: "Laden" }));
    await waitFor(() => expect(setup.level("thorough")?.downloading).toBe(true));
    await settle();
    expect(help().textContent?.trim()).toMatch(/^„Gründlich“ wird geladen: \d+ %\.$/);
    expect(document.body.textContent).not.toMatch(claim);
    // What really happens on sending: the dialog after model_missing.
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    expect(screen.getByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" })).toBeTruthy();
    expect(field().value).toBe(QUESTION);
    mock.modelFiles.release();
  });

  test("KC82 KC49 the third level failed: the help line keeps cause, parts and offline package, no level answers", async () => {
    const mock = await storedWithoutModel();
    const ready = new Set([...mock.modelFiles.ofLevel("balanced")]);
    const only = mock.modelFiles.ofLevel("thorough").find((key) => !ready.has(key))!;
    mock.modelFiles.failChecksumOnce(only);
    await choose(/Gründlich/);
    await fireEvent.click(screen.getByRole("button", { name: "Laden" }));
    await waitFor(() => expect(setup.level("thorough")?.error).toBeTruthy(), { timeout: 10_000 });
    await settle();
    expect(help().classList.contains("danger")).toBe(true);
    expect(help().textContent).toContain("Laden von „Gründlich“ fehlgeschlagen:");
    expect(help().textContent).toContain(
      "Die geladenen Teile bleiben, ein neuer Versuch setzt dort fort.",
    );
    expect(help().textContent).toContain("Offline-Paket in den Einstellungen");
    expect(help().textContent).not.toContain("antwortet");
    expect(document.body.textContent).not.toMatch(claim);
    const menu = await openMenu();
    expect(within(menu).getByRole("menuitemradio", { name: /Gründlich/ }).textContent).toContain(
      "Wählen, um es erneut zu versuchen.",
    );
  }, 20_000);
});

describe("Erneut fragen", () => {
  test("KC51 KC52 the menu offers every level, a fast answer first 'Gründlicher beantworten'; the level stays", async () => {
    const mock = await start({ scenarios: ["all-levels"] });
    const askAgain = vi.spyOn(mock.api, "ask_again");
    await openChat("chat-example");
    await fireEvent.click(screen.getByRole("button", { name: /Erneut fragen/ }));
    const menu = screen.getByRole("menu", { name: "Erneut fragen in Stufe" });
    expect(
      within(menu)
        .getAllByRole("menuitem")
        .map((i) => i.querySelector(".name")?.textContent),
    ).toEqual(["Gründlicher beantworten", "In „Schnell“", "In „Ausgewogen“", "In „Gründlich“"]);
    await fireEvent.click(within(menu).getByRole("menuitem", { name: /Gründlicher beantworten/ }));
    expect(askAgain).toHaveBeenCalledWith(
      expect.objectContaining({ chat: "chat-example", turn: 0, level: "balanced" }),
      expect.any(Function),
    );
    await waitFor(() => expect(screen.getAllByRole("article")).toHaveLength(2));
    await waitFor(
      () => {
        const html = screen.getAllByRole("article")[1].textContent;
        expect(html).toContain("Ausgewogen ·");
      },
      { timeout: 5_000 },
    );
    // The old answer stays; the chosen level is unchanged (ADR 0065).
    expect(screen.getAllByRole("article")[0].querySelector("footer")?.textContent).toContain(
      "Schnell",
    );
    expect(answerLevel.current).toBe("fast");
    expect((await mock.api.get_settings()).answer_level).toBe("fast");
    expect(stepper().textContent).toBe("Schnell");
  });

  test("KC51 at a failed turn, with the index over all stored turns; a missing model offers the download", async () => {
    const mock = await start();
    const askAgain = vi.spyOn(mock.api, "ask_again");
    await openChat("chat-interrupted");
    const failed = screen.getAllByRole("article")[1];
    await fireEvent.click(within(failed).getByRole("button", { name: /Erneut fragen/ }));
    await fireEvent.click(screen.getByRole("menuitem", { name: /In „Schnell“/ }));
    expect(askAgain).toHaveBeenCalledWith(
      expect.objectContaining({ chat: "chat-interrupted", turn: 1, level: "fast" }),
      expect.any(Function),
    );
    await waitFor(() => expect(screen.getAllByRole("article")).toHaveLength(3));
    await waitFor(() =>
      expect(screen.getAllByRole("article")[2].querySelector("footer")).toBeTruthy(),
    );
    // Gründlich has no model here: the dialog, without switching the level afterwards.
    const cancelled = screen.getAllByRole("article")[0];
    await fireEvent.click(within(cancelled).getByRole("button", { name: /Erneut fragen/ }));
    await fireEvent.click(screen.getByRole("menuitem", { name: /In „Gründlich“/ }));
    const dialog = screen.getByRole("dialog", { name: "„Gründlich“ zuerst laden?" });
    expect(dialog.textContent).toContain("Danach kannst du die Frage erneut stellen.");
    expect(askAgain).toHaveBeenCalledTimes(1);
  });

  test("KC53 KC52 after 30 s without text in Ausgewogen: 'Abbrechen und schnell fragen' asks fast", async () => {
    vi.useFakeTimers({ toFake: ["setInterval", "Date"] });
    const mock = await start({ scenarios: ["all-levels"], answerLevel: "balanced" });
    const { asked, cancelled } = scripted(mock);
    await fireEvent.input(field(), { target: { value: QUESTION } });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    asked[0].emit({ type: "status", phase: "starting_model", progress: null });
    const offer = () => screen.queryByRole("button", { name: "Abbrechen und schnell fragen" });
    vi.advanceTimersByTime(29_000);
    await settle(1);
    expect(offer()).toBeNull();
    expect(screen.getByRole("button", { name: "Abbrechen" })).toBeTruthy();
    vi.advanceTimersByTime(1_000);
    await settle(1);
    expect(offer()).toBeTruthy();
    await fireEvent.click(offer()!);
    expect(cancelled).toHaveLength(1);
    asked[0].finish(answerOf("", [], { level: "balanced", outcome: { type: "cancelled" } }));
    await settle();
    expect(asked).toHaveLength(2);
    expect(asked[1].args).toMatchObject({ question: QUESTION, level: "fast" });
    expect(answerLevel.current).toBe("balanced");
    expect((await mock.api.get_settings()).answer_level).toBe("balanced");
  });

  test("KC53 no way out once text came, and never in Schnell", async () => {
    vi.useFakeTimers({ toFake: ["setInterval", "Date"] });
    const mock = await start({ scenarios: ["all-levels"], answerLevel: "balanced" });
    const { asked } = scripted(mock);
    await fireEvent.input(field(), { target: { value: QUESTION } });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    asked[0].emit({ type: "delta", text: "Text" });
    vi.advanceTimersByTime(40_000);
    await settle(1);
    expect(screen.queryByRole("button", { name: "Abbrechen und schnell fragen" })).toBeNull();
    asked[0].finish(answerOf("Text"));
    await settle();
    await choose(/Schnell/);
    await fireEvent.input(field(), { target: { value: `${QUESTION} 2` } });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    vi.advanceTimersByTime(60_000);
    await settle(1);
    expect(asked[1].args.level).toBe("fast");
    expect(screen.queryByRole("button", { name: "Abbrechen und schnell fragen" })).toBeNull();
  });
});

test("KC66 the chat area shows English with ?mock-lang=en", async () => {
  await start({ language: "en" });
  i18n.language = "en";
  await settle();
  expect(stepper().textContent).toBe("Fast");
  expect(screen.getByLabelText("Question to your documents")).toBeTruthy();
  expect(screen.getByRole("button", { name: "Send" })).toBeTruthy();
  expect(help().textContent).toContain("Fast: the answer usually starts within half a minute.");
});
