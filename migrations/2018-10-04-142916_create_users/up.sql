CREATE TABLE users (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    authorization_token VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('users');
