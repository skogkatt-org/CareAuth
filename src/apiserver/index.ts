import Koa from 'koa';
import bodyParser from 'koa-bodyparser';

import { writeErrorTo, INTERNAL_SERVER_ERROR, ENDPOINT_NOT_FOUND } from './errors.js';
import { router } from './router.js';

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

// Let the requests be handled by routers.
koa.use(router.routes());

// Handle if we miss the requests.
koa.use((ctx: Koa.Context, next: () => Promise<any>) => {
  writeErrorTo(ctx, ENDPOINT_NOT_FOUND)
  return next();
})

export const apiserver = koa;
