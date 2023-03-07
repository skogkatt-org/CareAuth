import Pg from "pg";
import type { QueryResult } from "pg";
import { z } from "zod";
import bcrypt from "bcrypt";
import jwt from 'jsonwebtoken';

const client = new Pg.Pool({
  connectionString: process.env.POSTGRES_URL,
});
await client.connect();

export class App {
  constructor() { }

  async listRoles(): Promise<Roles> {
    const res: QueryResult<Role> = await client.query("SELECT * FROM roles");
    return { roles: res.rows };
  }

  async createRole(role: RoleInput): Promise<Role> {
    const res: QueryResult<Role> = await client.query(
      "INSERT INTO roles(name) VALUES($1) RETURNING *",
      [role.name]
    );
    return res.rows[0];
  }

  async getRole(id: number): Promise<Role> {
    const res: QueryResult<Role> = await client.query(
      "SELECT * FROM roles WHERE id = $1",
      [id]
    );
    return res.rows[0];
  }

  async updateRole(role: Role): Promise<Role> {
    const res: QueryResult<Role> = await client.query(
      "UPDATE roles SET name = $1 WHERE id = $2 RETURNING *",
      [role.name, role.id]
    );
    return res.rows[0];
  }

  async deleteRole(id: number) {
    await client.query("DELETE FROM roles WHERE id = $1", [id]);
  }

  async listUsers(): Promise<Users> {
    const res: QueryResult<User> = await client.query("SELECT * FROM users");
    return { users: res.rows };
  }

  async createUser(user: UserInput): Promise<User> {
    const res: QueryResult<User> = await client.query(
      "INSERT INTO users(name, hashed_password) VALUES($1, $2) RETURNING *",
      [user.name, await hash_password(user.password)]
    );
    return res.rows[0];
  }

  async getUser(id: number): Promise<User> {
    const res: QueryResult<User> = await client.query(
      "SELECT * FROM users WHERE id = $1",
      [id]
    );
    return res.rows[0];
  }

  async updateUser(user: User): Promise<User> {
    const res: QueryResult<User> = await client.query(
      "UPDATE users SET name = $1, hashed_password = $2 WHERE id = $3 RETURNING *",
      [user.name, await hash_password(user.password), user.id]
    );
    return res.rows[0];
  }

  async deleteUser(id: number) {
    await client.query("DELETE FROM users WHERE id = $1", [id]);
  }

  /**
   * @param login The argument to login.
   * @returns The Login object.
   * @throws If the the password is not corrent or something else.
   */
  async login(username: string, password: string): Promise<Login> {
    const res: QueryResult<User> = await client.query(
      "SELECT * FROM users WHERE name = $1",
      [username]
    );
    let user = res.rows[0];
    if (await bcrypt.compare(password, user.hashed_password)) {
      const secert = process.env.JWT_SECRET;
      if (secert === undefined) throw new Error("variable JWT_SECRET not found");
      return { token: jwt.sign({ data: user }, secert) };
    }
    throw new Error("invalid password");
  }

  async verify(token: string): Promise<VerifiedLogin> {
    const secert = process.env.JWT_SECRET;
    if (secert === undefined) throw new Error("variable JWT_SECRET not found");
    return jwt.verify(token, secert) as VerifiedLogin
  }
};

async function hash_password(password: string): Promise<string> {
  return await bcrypt.hash(password, 12);
}

export interface Role {
  id: number;
  name: string;
};

export const roleInputSchema = z.object({
  name: z.string(),
});

export type RoleInput = z.infer<typeof roleInputSchema>;

export interface Roles {
  roles: Role[];
}

export interface User {
  id: number;
  name: string;
  password: string;
  hashed_password: string;
}

export const userInputSchema = z.object({
  name: z.string(),
  password: z.string(),
});

export type UserInput = z.infer<typeof userInputSchema>;

export interface Users {
  users: User[];
}

export const loginInputSchema = z.object({
  username: z.string(),
  password: z.string(),
});

export type LoginInput = z.infer<typeof loginInputSchema>;

export const loginSchema = z.object({
  token: z.string(),
})

export type Login = z.infer<typeof loginSchema>;

export interface VerifiedLogin {
  data: {
    user: User,
  }
}
