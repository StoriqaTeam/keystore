use std::cell::RefCell;

use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;
use futures_cpupool::CpuPool;

use super::error::*;
use prelude::*;

thread_local! {
    pub static DB_CONN: RefCell<Option<PgPooledConnection>> = RefCell::new(None)
}

pub trait DbExecutor: Clone + Send + Sync + 'static {
    fn execute<F, T>(&self, f: F) -> Box<Future<Item = T, Error = Error> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, Error> + Send + 'static;
    fn execute_transaction<F, T>(&self, f: F) -> Box<Future<Item = T, Error = Error> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, Error> + Send + 'static;
}

#[derive(Clone)]
pub struct DbExecutorImpl {
    db_pool: PgPool,
    db_thread_pool: CpuPool,
}

impl DbExecutorImpl {
    pub fn new(db_pool: PgPool, db_thread_pool: CpuPool) -> Self {
        Self { db_pool, db_thread_pool }
    }
}

impl DbExecutor for DbExecutorImpl {
    fn execute<F, T>(&self, f: F) -> Box<Future<Item = T, Error = Error> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, Error> + Send + 'static,
    {
        let db_pool = self.db_pool.clone();
        Box::new(self.db_thread_pool.spawn_fn(move || {
            DB_CONN.with(move |maybe_conn_cell| -> Result<T, Error> {
                {
                    let mut maybe_conn = maybe_conn_cell.borrow_mut();
                    if maybe_conn.is_none() {
                        match db_pool.get() {
                            Ok(conn) => *maybe_conn = Some(conn),
                            Err(e) => return Err(ectx!(err e, ErrorSource::R2D2, ErrorKind::Internal)),
                        }
                    }
                }
                f()
            })
        }))
    }

    fn execute_transaction<F, T>(&self, f: F) -> Box<Future<Item = T, Error = Error> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, Error> + Send + 'static,
    {
        let db_pool = self.db_pool.clone();
        Box::new(self.db_thread_pool.spawn_fn(move || {
            DB_CONN.with(move |maybe_conn_cell| -> Result<T, Error> {
                {
                    let mut maybe_conn = maybe_conn_cell.borrow_mut();
                    if maybe_conn.is_none() {
                        match db_pool.get() {
                            Ok(conn) => *maybe_conn = Some(conn),
                            Err(e) => return Err(ectx!(err e, ErrorSource::R2D2, ErrorKind::Internal)),
                        }
                    }
                }
                with_tls_connection(move |conn| {
                    let mut e: Error = ErrorKind::Internal.into();
                    conn.transaction(|| {
                        f().map_err(|err| {
                            e = err;
                            DieselError::RollbackTransaction
                        })
                    }).map_err(|_| e)
                })
            })
        }))
    }
}

pub fn with_tls_connection<F, T>(f: F) -> Result<T, Error>
where
    F: FnOnce(&PgConnection) -> Result<T, Error>,
{
    DB_CONN.with(|maybe_conn_cell| -> Result<T, Error> {
        let conn: PgPooledConnection;
        {
            let mut maybe_conn = maybe_conn_cell.borrow_mut();
            if maybe_conn.is_none() {
                return Err(ectx!(err ErrorKind::Internal, ErrorContext::Connection, ErrorKind::Internal));
            }
            conn = maybe_conn.take().unwrap();
        }
        let res = f(&conn);
        {
            let mut maybe_conn = maybe_conn_cell.borrow_mut();
            *maybe_conn = Some(conn);
        }
        res
    })
}
