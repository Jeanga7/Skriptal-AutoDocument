-- Add migration script here
CREATE TABLE revoked_tokens (
    id SERIAL PRIMARY KEY,
    token TEXT NOT NULL UNIQUE,
    revoked_at TIMESTAMP DEFAULT NOW()
);
