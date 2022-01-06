interface User {
  first_name: string;
  last_name: string;
  age: number;
  active: boolean;
}

export const run = (user: User): User => {
  if (user.age < 13) {
    user.active = false;
  }

  return user;
};
