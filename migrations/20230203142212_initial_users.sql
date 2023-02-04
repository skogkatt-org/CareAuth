-- migrate:up
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

CREATE UNIQUE INDEX unique_user_name ON users (
    name
);

-- migrate:down
DROP INDEX unique_user_name;

DROP TABLE users;
