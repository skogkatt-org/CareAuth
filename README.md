# CareAuth: A simple authentication service

## Dependencies

Running Environment:

- PostgreSQL: The database to store data.

Devlopment:

- Rust nightly: This project is write by Rust.
- sqlx cli: We use sqlx cli to manage the database migration.

## Guide to Test

```shell
# Run the PostgreSQL by Docker.
$ bash scripts/testing_postgres.sh
# Run the application by Rust's cargo (and using the test database).
$ bash scripts/testing_run.sh
```
