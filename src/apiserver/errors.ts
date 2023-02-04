import Koa from 'koa';
import _ from 'lodash';

export interface ServerError {
  status: number,
  error: {
    code: string,
    description: string,
  }
}

export const ENDPOINT_NOT_FOUND: ServerError = {
  status: 404,
  error: {
    code: 'endpoint_not_found',
    description: 'endpoint not found',
  }
};

export const INVALID_ARGUMENT: ServerError = {
  status: 400,
  error: {
    code: 'invalid_argument',
    description: 'invalid argument',
  }
};

export const INTERNAL_SERVER_ERROR: ServerError = {
  status: 500,
  error: {
    code: 'internal_server_error',
    description: 'internal server error',
  }
};

export function writeErrorTo(ctx: Koa.Context, error: ServerError): void;
export function writeErrorTo(ctx: Koa.Context, error: ServerError, description: string): void;
export function writeErrorTo(ctx: Koa.Context, error: ServerError, description?: string) {
  ctx.status = error.status;
  if (description !== undefined) {
    error = _.cloneDeep(error)
    error.error.description = description
  }
  ctx.body = { error: error.error };
}