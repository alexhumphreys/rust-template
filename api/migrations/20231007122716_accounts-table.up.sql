CREATE TABLE IF NOT EXISTS accounts (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    name TEXT NOT NULL,
    credential TEXT NOT NULL
);
