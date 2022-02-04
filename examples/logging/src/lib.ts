import { log } from "@suborbital/js";

export const run = (user: string): string => {
  let message = "Hello, " + user;

  log.info(message);

  return message;
};
