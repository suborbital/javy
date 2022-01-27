import { Env } from "./env";

interface User {
  first_name: string;
  last_name: string;
  age: number;
  active: boolean;
}

export const env = new Env();

export const run = (user: User, ident: number): User => {
  user.first_name = "Modified first name";
  user.last_name = "Modified last name";

  return user;
};
