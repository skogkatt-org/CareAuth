#!/usr/bin/env bash

docker run --name test-postgres \
    -p 5432:5432 \
    -e POSTGRES_USER=test -e POSTGRES_PASSWORD=test -e POSTGRES_DB=test \
    -d postgres