table! {
    keys (id) {
        id -> Uuid,
        private_key -> Varchar,
        blockchain_address -> Varchar,
        currency -> Varchar,
        owner_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        authentication_token -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(keys -> users (owner_id));

allow_tables_to_appear_in_same_query!(
    keys,
    users,
);
