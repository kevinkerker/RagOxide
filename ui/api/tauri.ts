// Backend inside the desktop app: every command becomes `invoke(name, { args, on_event })`.
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { isApiError, type Api, type Backend } from "./backend";
import { COMMAND_NAMES, type ApiError, type DocumentFile, type DocumentFileInfo } from "./bindings";

export { isTauri };

/**
 * Every rejection as an `ApiError`. Commands reject with one; Tauri itself rejects with a
 * plain string when it refuses the call before the command runs (arguments that do not
 * deserialise, an unknown command, a missing permission). That is a wrong call, so it
 * becomes `invalid_input` with Tauri's text.
 */
export function toApiError(error: unknown): ApiError {
  if (isApiError(error)) return error;
  const message = error instanceof Error ? error.message : String(error);
  return { kind: "invalid_input", message };
}

/**
 * `document_file` answers with a raw body: `u32` little-endian length, the JSON of
 * `DocumentFileInfo`, then the file bytes (see `DocumentFile` in app/src/api.rs).
 */
export function decodeDocumentFile(raw: ArrayBuffer): DocumentFile {
  const length = new DataView(raw).getUint32(0, true);
  const info = JSON.parse(
    new TextDecoder().decode(new Uint8Array(raw, 4, length)),
  ) as DocumentFileInfo;
  return { ...info, bytes: raw.slice(4 + length) };
}

/** Commands whose raw answer needs decoding before it has its declared type. */
const DECODERS: Partial<Record<string, (raw: never) => unknown>> = {
  document_file: decodeDocumentFile,
};

export function createTauriBackend(): Backend {
  const api: Record<string, (args?: unknown, onEvent?: (event: unknown) => void) => unknown> = {};
  for (const name of COMMAND_NAMES) {
    const decode = DECODERS[name];
    api[name] = async (args, onEvent) => {
      const payload: Record<string, unknown> = {};
      if (args !== undefined) payload.args = args;
      if (onEvent) {
        const channel = new Channel<unknown>();
        channel.onmessage = onEvent;
        payload.on_event = channel;
      }
      let result: unknown;
      try {
        result = await invoke(name, payload);
      } catch (error) {
        throw toApiError(error);
      }
      return decode ? decode(result as never) : result;
    };
  }
  return {
    kind: "tauri",
    api: api as unknown as Api,
    listen: (event, handler) =>
      listen<Parameters<typeof handler>[0]>(event, (message) => handler(message.payload)),
  };
}
