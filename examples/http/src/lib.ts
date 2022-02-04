import { log, http } from "@suborbital/js";

export const run = (input: string): string => {
  log.info("Fetching `https://httpbin.org/get`");

  const response = http.get("https://httpbin.org/get");

  log.info(response.text());

  return input;
};
