CREATE TABLE keys (
    id UUID PRIMARY KEY,
    private_key VARCHAR NOT NULL,
    blockchain_address VARCHAR NOT NULL,
    currency VARCHAR NOT NULL,
    owner_id UUID NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX keys_private_key_idx ON keys (private_key, currency);
CREATE UNIQUE INDEX keys_blockchain_address_idx ON keys (blockchain_address, currency);
SELECT diesel_manage_updated_at('keys');
