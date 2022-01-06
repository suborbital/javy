import { PaymentMethodsAPI } from "@shopify/scripts-checkout-apis-ts";

type Payload = PaymentMethodsAPI.Payload;
type Output = PaymentMethodsAPI.Output;

export const run = (payload: Payload): Output => {
  return {
    sortResponse: {
      proposedOrder: payload.input.paymentMethods,
    },
    filterResponse: null,
    renameResponse: null,
  };
};
