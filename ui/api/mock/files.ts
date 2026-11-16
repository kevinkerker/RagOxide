// Source files for `document_file` in the mock: the example documents exist only as
// parsed JSON, so the mock renders a PDF with each element's text at its anchor box and
// a blank PNG of the page size for images. Good enough to show pages and highlight
// boxes; real layout comes from the test corpus.
import type { Anchor, BoundingBox, Document } from "../bindings";

const FONT_SIZE = 10;
const LINE_HEIGHT = 12;

/** Characters outside Latin-1 that WinAnsiEncoding places at 0x80-0x9F. */
const WIN_ANSI: Record<string, number> = {
  "€": 0x80,
  "‚": 0x82,
  "„": 0x84,
  "…": 0x85,
  "‘": 0x91,
  "’": 0x92,
  "“": 0x93,
  "”": 0x94,
  "•": 0x95,
  "–": 0x96,
  "—": 0x97,
};

/** Text as a PDF string literal in WinAnsiEncoding. */
function pdfString(text: string): string {
  let out = "(";
  for (const char of text) {
    const code = char.codePointAt(0)!;
    if (char === "(" || char === ")" || char === "\\") out += `\\${char}`;
    else if (code >= 0x20 && code < 0x7f) out += char;
    else {
      const byte = WIN_ANSI[char] ?? (code >= 0xa0 && code <= 0xff ? code : 0x3f);
      out += `\\${byte.toString(8).padStart(3, "0")}`;
    }
  }
  return `${out})`;
}

/** Splits text into lines of at most `width` points at the approximate Helvetica width. */
function wrap(text: string, width: number): string[] {
  const maxChars = Math.max(8, Math.floor(width / (FONT_SIZE * 0.5)));
  const lines: string[] = [];
  let line = "";
  for (const word of text.replace(/\s+/g, " ").trim().split(" ")) {
    if (line && line.length + 1 + word.length > maxChars) {
      lines.push(line);
      line = word;
    } else {
      line = line ? `${line} ${word}` : word;
    }
  }
  if (line) lines.push(line);
  return lines;
}

type PageBox = { page: number; bbox: BoundingBox };

function pageBoxes(anchors: Anchor[]): PageBox[] {
  return anchors.flatMap((a) => (a.type === "page" ? [{ page: a.page, bbox: a.bbox }] : []));
}

/** Text drawing operations per page number. */
function layout(document: Document): Map<number, string[]> {
  const pages = new Map<number, string[]>();
  const height = new Map(document.pages.map((p) => [p.page, p.height]));
  const draw = (text: string, boxes: PageBox[]) => {
    const lines = boxes.length > 0 ? wrap(text, boxes[0].bbox.right - boxes[0].bbox.left) : [];
    let next = 0;
    boxes.forEach((box, index) => {
      const capacity = Math.max(1, Math.floor((box.bbox.bottom - box.bbox.top) / LINE_HEIGHT));
      const take = index === boxes.length - 1 ? lines.length - next : capacity;
      const ops = pages.get(box.page) ?? [];
      lines.slice(next, next + take).forEach((line, i) => {
        const y = (height.get(box.page) ?? 842) - box.bbox.top - FONT_SIZE - i * LINE_HEIGHT;
        ops.push(`BT /F1 ${FONT_SIZE} Tf ${box.bbox.left} ${y} Td ${pdfString(line)} Tj ET`);
      });
      pages.set(box.page, ops);
      next += take;
    });
  };
  for (const element of document.elements) {
    if (element.kind.type === "table" && element.kind.cells.some((c) => c.anchor)) {
      for (const cell of element.kind.cells) {
        if (cell.anchor) draw(cell.text, pageBoxes([cell.anchor]));
      }
    } else {
      draw(element.text, pageBoxes(element.anchors));
    }
  }
  return pages;
}

function bytesOf(text: string): Uint8Array<ArrayBuffer> {
  const bytes = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) bytes[i] = text.charCodeAt(i) & 0xff;
  return bytes;
}

/** A PDF with the pages of `document` and every element's text at its boxes. */
export function pdfOf(document: Document): ArrayBuffer {
  const ops = layout(document);
  const objects: string[] = [];
  const pageIds = document.pages.map((_, i) => 4 + i * 2);
  objects[1] = "<< /Type /Catalog /Pages 2 0 R >>";
  objects[2] = `<< /Type /Pages /Kids [${pageIds.map((id) => `${id} 0 R`).join(" ")}] /Count ${pageIds.length} >>`;
  objects[3] = "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>";
  document.pages.forEach((page, i) => {
    const content = (ops.get(page.page) ?? []).join("\n");
    objects[pageIds[i]] =
      `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${page.width} ${page.height}] ` +
      `/Resources << /Font << /F1 3 0 R >> >> /Contents ${pageIds[i] + 1} 0 R >>`;
    objects[pageIds[i] + 1] = `<< /Length ${content.length} >>\nstream\n${content}\nendstream`;
  });
  let out = "%PDF-1.4\n";
  const offsets: number[] = [];
  for (let id = 1; id < objects.length; id++) {
    offsets[id] = out.length;
    out += `${id} 0 obj\n${objects[id]}\nendobj\n`;
  }
  const xref = out.length;
  out += `xref\n0 ${objects.length}\n0000000000 65535 f \n`;
  for (let id = 1; id < objects.length; id++) {
    out += `${String(offsets[id]).padStart(10, "0")} 00000 n \n`;
  }
  out += `trailer\n<< /Size ${objects.length} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return bytesOf(out).buffer;
}

const CRC_TABLE = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});

function crc32(bytes: Uint8Array): number {
  let c = 0xffffffff;
  for (const byte of bytes) c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function pngChunk(type: string, data: Uint8Array): Uint8Array {
  const out = new Uint8Array(12 + data.length);
  const view = new DataView(out.buffer);
  view.setUint32(0, data.length);
  out.set(bytesOf(type), 4);
  out.set(data, 8);
  view.setUint32(8 + data.length, crc32(out.subarray(4, 8 + data.length)));
  return out;
}

async function zlib(data: Uint8Array<ArrayBuffer>): Promise<Uint8Array> {
  const stream = new Response(data).body!.pipeThrough(new CompressionStream("deflate"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

/** A white 8-bit greyscale PNG of the given size. */
export async function blankPng(width: number, height: number): Promise<ArrayBuffer> {
  const header = new Uint8Array(13);
  const view = new DataView(header.buffer);
  view.setUint32(0, width);
  view.setUint32(4, height);
  header[8] = 8; // bit depth
  header[9] = 0; // greyscale
  const raw = new Uint8Array(height * (width + 1)).fill(0xff);
  for (let row = 0; row < height; row++) raw[row * (width + 1)] = 0; // filter: none
  const parts = [
    new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]),
    pngChunk("IHDR", header),
    pngChunk("IDAT", await zlib(raw)),
    pngChunk("IEND", new Uint8Array(0)),
  ];
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out.buffer;
}
