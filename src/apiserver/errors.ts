import Koa from 'koa';
import _ from 'lodash';

export interface ServerError {
  code: string,
  desc: string,
}

export const ENDPOINT_NOT_FOUND: ServerError = {
  code: 'ENDPOINT_NOT_FOUND',
  desc: 'endpoint not found',
};

export const INVALID_ARGUMENT: ServerError = {
  code: 'INVALID_ARGUMENT',
  desc: 'invalid argument',
};

export const INTERNAL_SERVER_ERROR: ServerError = {
  code: 'INTERNAL',
  desc: 'internal server error',
};

export function writeErrorTo(ctx: Koa.Context, error: ServerError): void;
export function writeErrorTo(ctx: Koa.Context, error: ServerError, description: string): void;
export function writeErrorTo(ctx: Koa.Context, error: ServerError, description?: string) {
  ctx.status = 200;
  if (description !== undefined) {
    error = _.cloneDeep(error)
    error.desc = description
  }
  ctx.body = { type: "ERROR", result: error };
}