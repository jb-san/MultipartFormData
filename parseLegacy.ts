export type Part = {
  contentDispositionHeader: string;
  contentTypeHeader: string;
  part: number[];
};
interface BaseFileEntry {
  data: Blob;
}
export type FileEntry = BaseFileEntry & {
  [key: string]: string | Blob; // Allow both string and Blob types
};
export enum ParsingState {
  INIT,
  READING_HEADERS,
  READING_DATA,
  READING_PART_SEPARATOR,
}
export function process(
  multipartBodyBuffer: ArrayBuffer,
  boundary: string
): FileEntry[] {
  const decoder = new TextDecoder("utf-8");
  const bytes = new Uint8Array(multipartBodyBuffer);

  let lastline = "";
  let contentDispositionHeader = "";
  let contentTypeHeader = "";
  let state: ParsingState = ParsingState.INIT;
  let buffer: number[] = [];
  const allParts: FileEntry[] = [];

  let currentPartHeaders: string[] = [];

  for (let i = 0; i < bytes.length; i++) {
    const oneByte: number = bytes[i];
    const prevByte: number | null = i > 0 ? bytes[i - 1] : null;
    // 0x0a => \n
    // 0x0d => \r
    const newLineDetected: boolean = oneByte === 0x0a && prevByte === 0x0d;
    const newLineChar: boolean = oneByte === 0x0a || oneByte === 0x0d;

    if (!newLineChar) lastline += decoder.decode(new Uint8Array([oneByte]));
    if (ParsingState.INIT === state && newLineDetected) {
      // searching for boundary
      if ("--" + boundary === lastline) {
        state = ParsingState.READING_HEADERS; // found boundary. start reading headers
      }
      lastline = "";
    } else if (ParsingState.READING_HEADERS === state && newLineDetected) {
      // parsing headers. Headers are separated by an empty line from the content. Stop reading headers when the line is empty
      if (lastline.length) {
        currentPartHeaders.push(lastline);
      } else {
        // found empty line. search for the headers we want and set the values

        for (const h of currentPartHeaders) {
          if (h.toLowerCase().startsWith("content-disposition:")) {
            contentDispositionHeader = h;
          } else if (h.toLowerCase().startsWith("content-type:")) {
            contentTypeHeader = h;
          }
        }
        state = ParsingState.READING_DATA;
        buffer = [];
      }
      lastline = "";
    } else if (ParsingState.READING_DATA === state) {
      // parsing data
      if (lastline.length > boundary.length + 4) {
        lastline = ""; //mem save
      }
      if ("--" + boundary === lastline) {
        const j = buffer.length - lastline.length;
        const part = buffer.slice(0, j - 1);

        const ppart = process_part({
          contentDispositionHeader,
          contentTypeHeader,
          part,
        });
        allParts.push(ppart);
        buffer = [];
        currentPartHeaders = [];
        lastline = "";
        state = ParsingState.READING_PART_SEPARATOR;
        contentDispositionHeader = "";
        contentTypeHeader = "";
      } else {
        buffer.push(oneByte);
      }
      if (newLineDetected) {
        lastline = "";
      }
    } else if (ParsingState.READING_PART_SEPARATOR === state) {
      if (newLineDetected) {
        state = ParsingState.READING_HEADERS;
      }
    }
  }

  return allParts;
}

function process_part(part: Part) {
  const header = part.contentDispositionHeader.split(";").splice(1);
  const data = header
    .map((h) => parseContentDispositionHeaderKeyValue(h))
    .reduce((acc, curr) => {
      acc[curr.key] = curr.value;
      return acc;
    }, {}) as FileEntry;
  const contentType = parseContentTypeHeader(part.contentTypeHeader);
  data["data"] = new Blob([new Uint8Array(part.part)], { type: contentType });
  return data;
}

function parseContentDispositionHeaderKeyValue(
  header: string
): Record<string, string> {
  const [key, value] = header.replace(/"/g, "").trim().split("=");
  return { key, value };
}
function parseContentTypeHeader(header: string): string {
  if (!header) return "";
  return header.split(":")[1].trim();
}
