import { log } from "@suborbital/runnable";

export const run = (user: string): string => {
  let message = "Hello, " + user;

  log.info(message);

  return message;
};
