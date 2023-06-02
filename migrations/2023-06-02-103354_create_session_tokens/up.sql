-- Your SQL goes here

CREATE TABLE session_tokens (
  token TEXT PRIMARY KEY,
  username TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  FOREIGN KEY (username) REFERENCES users (username)
)