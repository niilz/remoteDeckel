-- Your SQL goes here
CREATE TABLE payments (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  receipt_identifier VARCHAR,
  payed_amount MONEY NOT NULL,
  payed_at TIMESTAMP NOT NULL
);
