import "fast-text-encoding";

import { main } from "./";

declare var TextEncoder: any;
declare var TextDecoder: any;

const decoder = new TextDecoder();
const encoder = new TextEncoder();

const user = {
  first_name: "test",
  last_name: "user",
  age: 10,
  active: true,
};

const bytes = encoder.encode(JSON.stringify(user)).buffer;

describe("json", () => {
  it("marks young users as inactive", () => {
    const result = main(bytes);
    const json = JSON.parse(decoder.decode(result));
    expect(json.active).toEqual(false);
  });
});
