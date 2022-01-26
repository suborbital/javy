import "fast-text-encoding";
import { run, env } from "./lib";

declare var TextEncoder: any;
declare var TextDecoder: any;

const decoder = new TextDecoder();
const encoder = new TextEncoder();

export { env };

export const run_e = (payload: ArrayBuffer, ident: number) => {
  let input = JSON.parse(decoder.decode(payload));
  let result = JSON.stringify(run(input, ident));
  let output = encoder.encode(result);
  env.returnResult(output, ident);
};
