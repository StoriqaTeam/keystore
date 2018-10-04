table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        authorization_token -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
