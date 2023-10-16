-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL
);
