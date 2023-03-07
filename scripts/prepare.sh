#!/usr/bin/env bash

set -e

echo "Try to create a new database by docker..."
docker run --name test-postgres \
    -p 5432:5432 \
    -e POSTGRES_USER=test -e POSTGRES_PASSWORD=test -e POSTGRES_DB=test \
    -d postgres

echo "Please follow it:"
echo
echo "  1. Run command `bash scripts/dbmate.sh up`"
