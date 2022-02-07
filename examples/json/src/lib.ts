interface User {
  first_name: string;
  last_name: string;
  age: number;
  active: boolean;
}

export const run = (user: User): User => {
  user.first_name = "Modified first name";
  user.last_name = "Modified last name";

  return user;
};
