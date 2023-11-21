-- Add up migration script here
CREATE INDEX idx_token
ON clients (token);
