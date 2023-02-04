-- migrate:up
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

CREATE UNIQUE INDEX unique_role_name ON roles (
    name
);

-- migrate:down
DROP INDEX unique_role_name;

DROP TABLE roles;
