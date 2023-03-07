import Koa from 'koa';
import bodyParser from 'koa-bodyparser';

import { writeErrorTo, INTERNAL_SERVER_ERROR, ENDPOINT_NOT_FOUND } from './errors.js';
import { App } from '../app.js';

const koa = new Koa();

// Global error handler - handle all errors we missed.
koa.use(async (ctx: Koa.Context, next: () => Promise<any>) => {
  try {
    return await next();
  } catch(e) {
    writeErrorTo(
      ctx,
      INTERNAL_SERVER_ERROR,
      "internal server error: " + (e?.toString ? e.toString() : JSON.stringify(e)),
    )
    return;
  }
});

// Get the body by parser.
koa.use(bodyParser());

const app = new App();

function writeResponseTo(ctx: Koa.Context, result: any) {
  ctx.body = { type: "OK", result }
}

// Let the requests be handled by routers.
koa.use(async (ctx: Koa.Context, next: () => Promise<any>) => {
  const { method, param, context } = ctx.request.body as any;
  console.debug(ctx.request.body);
  switch (method) {
    case 'role::list':
      writeResponseTo(ctx, await app.listRoles())
      break
    case 'role::create':
      writeResponseTo(ctx, await app.createRole(param.role))
      break
    case 'role::get':
      writeResponseTo(ctx, await app.getRole(param.id))
      break
    case 'role::update':
      writeResponseTo(ctx, await app.updateRole(param.role));
      break
    case 'role::delete':
      writeResponseTo(ctx, await app.deleteRole(param.id));
      break
    case 'user::list':
      writeResponseTo(ctx, await app.listUsers());
      break
    case 'user::create':
      writeResponseTo(ctx, await app.createUser(param.user));
      break
    case 'user::get':
      writeResponseTo(ctx, await app.getUser(param.id));
      break
    case 'user::update':
      writeResponseTo(ctx, await app.updateUser(param.user));
      break
    case 'user::delete':
      writeResponseTo(ctx, await app.deleteUser(param.id));
      break
    case 'auth::login':
      writeResponseTo(ctx, await app.login(param.username, param.password));
      break
    case 'auth::verify':
      writeResponseTo(ctx, await app.verify(param.token));
      break
    default:
      return next();
  }
});

// Handle if we miss the requests.
koa.use((ctx: Koa.Context, next: () => Promise<any>) => {
  writeErrorTo(ctx, ENDPOINT_NOT_FOUND)
  return next();
})

export const apiserver = koa;
