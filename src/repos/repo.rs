use diesel::pg::PgConnection;

/// Products repository, responsible for handling products
pub struct Repo<'a> {
    pub db_conn: &'a PgConnection,
}
