ALTER TABLE clients
ADD user_id UUID NOT NULL;
ALTER TABLE clients
ADD token VARCHAR(128) NOT NULL;
ALTER TABLE clients
ADD CONSTRAINT fk_clients_users
FOREIGN KEY (user_id)
REFERENCES users (id);
