import { instantiate } from "./lib/rs_lib.generated.js";

const { parse_multipart } = await instantiate();
const filePath = "./rs_lib/mocks/c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7.response";
const uintArray = await Deno.readFile(filePath);

const resp = parse_multipart(uintArray,"c006a72d54394df978b69f41250ae264904000dd4fc39631e10080c96cd7")
console.log(resp)