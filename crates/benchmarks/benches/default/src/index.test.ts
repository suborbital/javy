import { run } from "./";
import * as input from "./input.json";

describe("default payment methods script", () => {
  it("returns the given payment methods", () => {
    const result = run(input as any);
    expect(result.sortResponse?.proposedOrder).toEqual(
      input.input.paymentMethods
    );
  });
});
