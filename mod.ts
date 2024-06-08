import { instantiate } from "./lib/rs_lib.generated.js";
import { process } from "./parseLegacy.ts";

type Entry = {
  content: string | Blob;
  meta?: Record<string, string>;
};
export class MultipartFormData {
  private _entries = new Map<string, Entry>();
  private boundary: string;

  constructor(boundary?: string) {
    if (boundary === undefined) {
      this.boundary = `${Date.now().toString(16)}`;
    } else {
      this.boundary = boundary;
    }
  }

  /**
   * Parses a multipart/form-data response into a MultiPartFormData object using the wasm module.
   * @param data
   * @param boundary
   */
  static async parse(data: ArrayBuffer, boundary: string) {
    const { parse_multipart } = await instantiate();
    console.time("parse");
    const { segments } = parse_multipart(new Uint8Array(data), boundary);
    const formData = new MultipartFormData(boundary);
    for (const entry of segments) {
      const headers = MultipartFormData.#extractParseHeaders(entry.headers);
      const { name, ...restMeta } = headers;
      formData.append(
        name,
        new Blob([new Uint8Array(entry.body as number[])], {
          type: restMeta["Content-Type"],
        }),
        restMeta
      );
    }
    console.timeEnd("parse");
    return formData;
  }
  /**
   * Parses a multipart/form-data response into a MultiPartFormData object using a pure JS parser.
   * THIS IS NOT RECOMMENDED FOR LARGE FILES and should only be used if WebAssembly is not available.
   * @param data
   * @param boundary
   */
  static async parseLegacy(data: ArrayBuffer, boundary: string) {
    console.time("parseLegacy");
    const files = process(new Uint8Array(data), boundary);

    const formData = new MultipartFormData();
    for (const entry of files) {
      const { name, data, ...restMeta } = entry;
      formData.append(name as string, data, restMeta as Record<string, string>);
    }
    console.timeEnd("parseLegacy");
    return formData;
  }

  static #extractParseHeaders(headers: { name: string; value: string }[]) {
    // headers
    let _headers: Record<string, string> = {};
    headers.forEach((header) => {
      const { name, value } = header;
      if (name.toLowerCase() === "content-disposition") {
        const meta = MultipartFormData.#extractContentDisposition(value);
        _headers = { ..._headers, ...meta };
      }
      _headers[name] = value;
    });
    return _headers;
  }
  static #extractContentDisposition(contentDisposition: string) {
    const [, ...rest] = contentDisposition.split(";");
    const meta = rest.reduce((acc, curr) => {
      const [name, value] = curr.split("=");
      return { ...acc, [name.trim()]: value.replaceAll(/"/g, "") };
    }, {});
    return meta;
  }
  /**
   * append
   * This method appends a new entry to the form data. while trying to conform to the browser FormData API,
   * it is not fully the same. According to the multipart/form-data spec, the content-disposition header can contain other values than filename and name.
   * so the meta parameter can be used to add additional values to the content-disposition header.
   * @param name used to identify the entry
   * @param value the entry value
   * @param meta additional values to add to the content-disposition header
   */
  append(name: string, value: string | Blob, meta?: Record<string, string>) {
    if (meta === undefined) {
      this._entries.set(name, { content: value });
      return;
    }
    this._entries.set(name, { content: value, meta });
  }
  /**
   * delete
   * This method deletes an entry from the form data.
   * @param name used to identify the entry
   */
  delete(name: string) {
    this._entries.delete(name);
  }

  /**
   * get
   * This method returns the entry with the specified name.
   * @param name used to identify the entry
   */
  get(name: string) {
    return this._entries.get(name);
  }
  /**
   * getAll
   * This method returns all entries with the specified name.
   * @param name used to identify the entry
   */
  getAll(name: string) {
    return this._entries.get(name);
  }
  /**
   * has
   * This method returns a boolean indicating if the specified entry exists.
   * @param name used to identify the entry
   */
  has(name: string) {
    return this._entries.has(name);
  }
  /**
   * set
   * This method sets the value of an existing entry in the form data object.
   * @param name used to identify the entry
   * @param value the entry value
   * @param meta additional values to add to the content-disposition header
   */
  set(name: string, value: string | Blob, meta?: Record<string, string>) {
    if (meta === undefined) {
      this._entries.set(name, { content: value });
      return;
    }
    this._entries.set(name, { content: value, meta });
  }
  /**
   * entries
   * This method returns an iterator allowing to go through all key/value pairs contained in this object.
   */
  entries() {
    return this._entries.entries();
  }
  /**
   * keys
   * This method returns an iterator allowing to go through all keys of the key/value pairs contained in this object.
   */
  keys() {
    return this._entries.keys();
  }
  /**
   * values
   * This method returns an iterator allowing to go through all values of the key/value pairs contained in this object.
   */
  values() {
    return this._entries.values();
  }
  /**
   * forEach
   * This method allows to execute a function for each key/value pair contained in this object.
   * @param callback
   */
  forEach(
    callback: (value: Entry, key: string, map: Map<string, Entry>) => void
  ) {
    this._entries.forEach(callback);
  }
  #getContentDispositionStrings = (value: Entry) => {
    const { content, meta, ...rest } = value;
    const strings = Object.entries(meta).map(
      ([key, value]) => `${key}="${value}"`
    );
    return strings.join("; ");
  };
  blob() {
    const multipartBody = [];
    for (const [key, value] of this.entries()) {
      let headers = `--${this.boundary}\r\n`;
      headers += `Content-Disposition: form-data; name="${key}" ${this.#getContentDispositionStrings(
        value
      )}\r\n`;
      //@ts-expect-error TODO: fix this type error
      switch (typeof value.content.type) {
        case "string":
          headers += `Content-Type: text/plain\r\n\r\n`;
          break;
        case undefined:
          headers += `Content-Type: text/plain\r\n\r\n`;
          break;
        default:
          //@ts-expect-error TODO: fix this type error
          headers += `Content-Type: ${value.content.type}\r\n\r\n`;
          break;
      }

      const headersBlob = new Blob([headers]);
      multipartBody.push(headersBlob, value.content, new Blob(["\r\n"]));
    }
    multipartBody.push(new Blob([`--${this.boundary}--\r\n`]));
    const bodyBlob = new Blob(multipartBody, {
      type: `multipart/form-data; boundary=${this.boundary}`,
    });

    return bodyBlob;
  }
  /**
   * [Symbol.iterator]
   * This method allows to iterate through the entries of the form data object.
   */
  [Symbol.iterator]() {
    return this._entries.entries();
  }
}
