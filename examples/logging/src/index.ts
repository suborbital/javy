import "fastestsmallesttextencoderdecoder-encodeinto/EncoderDecoderTogether.min.js";
import { run, env } from "./lib";

declare global {
  var TextEncoder: any;
  var TextDecoder: any;
}

const decoder = new TextDecoder();
const encoder = new TextEncoder();

export { env };

export const run_e = (payload: ArrayBuffer, ident: number) => {
  let input = decoder.decode(payload);
  let result = run(input, ident);
  let output = encoder.encode(result);
  env.returnResult(output, ident);
};
