import { expect, test, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {},
  invoke: (...args: unknown[]) => invoke(...args),
  isTauri: () => true,
}));

const { createTauriBackend, decodeDocumentFile, toApiError } = await import("./tauri");

test("decodes the raw answer of document_file into info and bytes", () => {
  // Same layout as DocumentFile::encode in app/src/api.rs.
  const header = new TextEncoder().encode(JSON.stringify({ content_hash: "ab", changed: true }));
  const bytes = new TextEncoder().encode("%PDF-1.4");
  const raw = new Uint8Array(4 + header.length + bytes.length);
  new DataView(raw.buffer).setUint32(0, header.length, true);
  raw.set(header, 4);
  raw.set(bytes, 4 + header.length);
  const file = decodeDocumentFile(raw.buffer);
  expect(file.content_hash).toBe("ab");
  expect(file.changed).toBe(true);
  expect(new TextDecoder().decode(file.bytes)).toBe("%PDF-1.4");
});

test("rejections of Tauri itself arrive as a typed error", async () => {
  const backend = createTauriBackend();
  // As Tauri rejects arguments that do not deserialise, before the command runs.
  invoke.mockRejectedValueOnce(
    "invalid args `args` for command `reveal_document`: missing field `id`",
  );
  await expect(backend.api.reveal_document({ id: "x" })).rejects.toEqual({
    kind: "invalid_input",
    message: "invalid args `args` for command `reveal_document`: missing field `id`",
  });
  // A command's own error passes unchanged.
  invoke.mockRejectedValueOnce({ kind: "not_found", message: "document x" });
  await expect(backend.api.open_document({ id: "x" })).rejects.toEqual({
    kind: "not_found",
    message: "document x",
  });
  expect(toApiError(new Error("boom"))).toEqual({ kind: "invalid_input", message: "boom" });
});
