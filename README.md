# CareAuth: A simple authentication service

## Dependencies

Running Environment:

- PostgreSQL: The database to store data.

Devlopment:

- Node.js: This project is write by TypeScript with pnpm (but we are trying to
  rewrite it by Rust in the future).
- dbmate: Which help us to manage database. (Use `./scripts/dbmate.sh` as a
  helpful wrapper)

## Guide to Run the Server for Testing

```shell
# Prepare the environment to run the server.
$ bash scripts/prepare.sh
# Run the server.
$ bash scripts/run.sh
```
