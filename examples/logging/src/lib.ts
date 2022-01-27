import { Env, LogLevel } from "./env";

export const env = new Env();

export const run = (user: string, ident: number): string => {
  let message = "Hello, " + user;

  env.logMsg(message, LogLevel.Info, ident);

  return message;
};
