-- migrate:up
ALTER TABLE users
  ADD COLUMN hashed_password VARCHAR(255) NOT NULL DEFAULT '';

-- migrate:down
-- Do nothing...
