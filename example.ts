// import { instantiate } from "./lib/rs_lib.generated.js";
import { MultipartFormData } from "./mod.ts";
// const { parse_multipart } = await instantiate();
const filePath =
  "./rs_lib/mocks/c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7.response";
const uintArray = await Deno.readFile(filePath);

// const resp = parse_multipart(uintArray,"c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7")
// console.log(resp)

const resp2 = await MultipartFormData.parse(
  uintArray,
  "c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7"
);
// console.log("resp", resp2);
const resp = await MultipartFormData.parseLegacy(
  uintArray,
  "c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7"
);
// console.log("resp", resp);
const text = await resp2.blob().text();
Deno.writeTextFile("output.txt", text);
console.log("resp", await resp.blob().text());
