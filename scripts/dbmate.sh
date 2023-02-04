#!/usr/bin/env bash

set -e

export DATABASE_URL="postgres://test:test@localhost:5432/test?sslmode=disable"
export DBMATE_MIGRATIONS_DIR="./migrations"
export DBMATE_SCHEMA_FILE="./migrations/schema.sql"

dbmate $@