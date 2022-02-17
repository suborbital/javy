import { log, http } from "@suborbital/runnable";

export const run = (input: string): string => {
  log.info("Fetching `https://httpbin.org/get`");

  // Accidentally left off the scheme. Oops!
  // Should see full JS stacktrace.
  const response = http.get("httpbin.org/get");

  log.info(response.text());

  return input;
};
