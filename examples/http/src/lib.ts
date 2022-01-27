import { Env, LogLevel, HttpMethod } from "./env";

export const env = new Env();

export const run = (input: string, ident: number): string => {
  env.logMsg("Fetching `https://httpbin.org/get`", LogLevel.Info, ident);

  let resultSize = env.fetchUrl(
    HttpMethod.Get,
    "https://httpbin.org/get",
    new Uint8Array(1),
    ident
  );
  // @ts-ignore
  const ptr0 = env._exports.canonical_abi_realloc(0, 0, 1, resultSize * 1);
  env.getFfiResult(ptr0, ident);

  let response = new Uint8Array(
    // @ts-ignore
    env._exports.memory.buffer,
    ptr0,
    resultSize * 1
  );

  env.logMsg(new TextDecoder().decode(response), LogLevel.Info, ident);

  return input;
};
