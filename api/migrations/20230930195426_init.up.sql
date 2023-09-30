-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    IF NOT EXISTS clients (
        id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
        name VARCHAR(255) NOT NULL UNIQUE
    );
