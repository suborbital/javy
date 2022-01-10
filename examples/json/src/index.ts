import "fast-text-encoding";
import { run } from "./lib";

declare var TextEncoder: any;
declare var TextDecoder: any;

const decoder = new TextDecoder();
const encoder = new TextEncoder();

export const run_e = (payload: ArrayBuffer): ArrayBuffer => {
  let input = JSON.parse(decoder.decode(payload));
  let output = JSON.stringify(run(input));
  return encoder.encode(output).buffer;
};
