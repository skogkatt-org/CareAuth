#!/usr/bin/env bash

set -e

docker run --name test-postgres \
    -p 5432:5432 \
    -e POSTGRES_USER=test -e POSTGRES_PASSWORD=test -e POSTGRES_DB=test \
    -d postgres

export DATABASE_URL="postgres://test:test@localhost:5432/test?sslmode=disable"
export DBMATE_MIGRATIONS_DIR="./migrations"
export DBMATE_SCHEMA_FILE="./migrations/schema.sql"
dbmate up
