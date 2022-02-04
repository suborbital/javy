import "fastestsmallesttextencoderdecoder-encodeinto/EncoderDecoderTogether.min.js";
import { setup, runnable } from "@suborbital/js";
import { run } from "./lib";

declare global {
  var TextEncoder: any;
  var TextDecoder: any;
}

const decoder = new TextDecoder();

export function run_e(payload: ArrayBuffer, ident: number) {
  // Imports will be injected by the runtime
  // @ts-ignore
  setup(this.imports, ident);

  const input = JSON.parse(decoder.decode(payload));
  const result = JSON.stringify(run(input));

  runnable.returnResult(result);
}
