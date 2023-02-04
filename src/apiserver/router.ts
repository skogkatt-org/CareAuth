import Koa from 'koa';
import Router, { type RouterContext } from '@koa/router';

import { App, type RoleInput, type Role, roleInputSchema, userInputSchema, UserInput, LoginInput, loginInputSchema, loginSchema, Login } from '../app.js';
import { writeErrorTo, INVALID_ARGUMENT} from './errors.js';

export const router = new Router();
const app = new App();

router
  .prefix('/api/v1')
  .get('/roles', listRoles)
  .post('/roles', createRole)
  .get('/roles/:id', getRole)
  .put('/roles/:id', updateRole)
  .del('/roles/:id', deleteRole)
  .get('/users', listUsers)
  .post('/users', createUser)
  .get('/users/:id', getUser)
  .put('/users/:id', updateUser)
  .del('/users/:id', deleteUser)
  .post('/login', createLogin)
  .post('/login\:verify', verifyLogin)

async function listRoles(ctx: Koa.Context) {
  ctx.body = await app.listRoles();
}

async function createRole(ctx: Koa.Context) {
  try {
    let role = checkedRoleInput(ctx.request.body);
    ctx.body = await app.createRole(role);
  } catch {
    writeErrorTo(ctx, INVALID_ARGUMENT);
  }
}

async function getRole(ctx: RouterContext) {
  ctx.body = await app.getRole(parseInt(ctx.params.id));
}

async function updateRole(ctx: RouterContext) {
  let role = checkedRoleInput(ctx.request.body);
  ctx.body = await app.updateRole(parseInt(ctx.params.id), role);
}

async function deleteRole(ctx: RouterContext) {
  let id = parseInt(ctx.params.id);
  await app.deleteRole(id);
  ctx.status = 204
}

async function listUsers(ctx: RouterContext) {
  ctx.body = await app.listUsers();
}

async function createUser(ctx: Koa.Context) {
  try {
    let user = checkedUserInput(ctx.request.body);
    ctx.body = await app.createUser(user);
  } catch {
    writeErrorTo(ctx, INVALID_ARGUMENT);
  }
}

async function getUser(ctx: RouterContext) {
  ctx.body = await app.getUser(parseInt(ctx.params.id));
}

async function updateUser(ctx: RouterContext) {
  let user = checkedUserInput(ctx.request.body);
  ctx.body = await app.updateUser(parseInt(ctx.params.id), user)
}

async function deleteUser(ctx: RouterContext) {
  let id = parseInt(ctx.params.id);
  await app.deleteUser(id);
  ctx.status = 204
}

async function createLogin(ctx: RouterContext) {
  ctx.body = await app.createLogin(checkedLoginInput(ctx.request.body));
}

async function verifyLogin(ctx: RouterContext) {
  ctx.body = await app.verifyLogin(checkedLogin(ctx.request.body));
}

function checkedRoleInput(json: any): RoleInput {
  return roleInputSchema.parse(json)
}

function checkedUserInput(json: any): UserInput {
  return userInputSchema.parse(json)
}

function checkedLoginInput(json: any): LoginInput {
  return loginInputSchema.parse(json)
}

function checkedLogin(json: any): Login {
  return loginSchema.parse(json)
}