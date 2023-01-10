# CareAuth: A simple authentication service

## Dependencies

Running Environment:

- PostgreSQL: The database to store data.

Devlopment:

- Rust nightly: This project is write by Rust.
- sqlx cli: We use sqlx cli to manage the database migration.

## Guide to Test

- You can run the PostgreSQL by docker:

  ```shell
  $ docker run --name test-postgres -p 5432:5432 -e POSTGRES_USER=test -e POSTGRES_PASSWORD=test -e POSTGRES_DB=test -d postgres
  ```

- Then run the application by Rust's cargo (use database `test` as user `test` with password `test`):

  ```shell
  POSTGRES_URL=postgres://test:test@localhost:5432/test cargo run
  ```