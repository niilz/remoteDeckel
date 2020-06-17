-- Start Postgres: pg_ctl -D /usr/local/var/postgres/ start
-- Create Database if not yet done: createdb remote_deckel
-- Configure .env for diesel: echo DATABASE_URL=postgres://niilz:spendenalarm@localhost/remote_deckel > .env
-- Launch diesel-cli: diesel setup
-- Configure user-table wich diesel cli: diesel migration generate create_users
-- Configure table here (up.sql) and drop (in down.sql)
-- Create user-table: diesel migration run
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name VARCHAR NOT NULL,
  drink_count SMALLINT NOT NULL default 0,
  price MONEY NOT NULL default 0.5,
  last_paid TIMESTAMP NOT NULL default CURRENT_TIMESTAMP,
  last_total MONEY NOT NULL default 0.0,
  total MONEY NOT NULL default 0.0
)
