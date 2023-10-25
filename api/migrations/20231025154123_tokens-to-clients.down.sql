ALTER TABLE clients
DROP CONSTRAINT fk_clients_users;
ALTER TABLE clients
DROP COLUMN user_id;
