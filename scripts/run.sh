#!/usr/bin/env bash

set -e

POSTGRES_URL=postgres://test:test@localhost:5432/test \
    JWT_SECRET=do-not-use-me-in-prod \
    pnpm run _run
