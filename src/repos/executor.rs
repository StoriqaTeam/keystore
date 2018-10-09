use std::cell::RefCell;

use diesel::pg::PgConnection;
use failure::Error as FailureError;
use futures_cpupool::CpuPool;

use super::error::*;
use prelude::*;

thread_local! {
    pub static DB_CONN: RefCell<Option<PgPooledConnection>> = RefCell::new(None)
}

pub trait DbExecutor: Clone + Send + Sync + 'static {
    fn execute<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Fail;
    fn execute_transaction<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Fail;
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
    fn execute<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Fail,
    {
        let db_pool = self.db_pool.clone();
        Box::new(self.db_thread_pool.spawn_fn(move || {
            DB_CONN.with(move |maybe_conn_cell| -> Result<T, E> {
                {
                    let mut maybe_conn = maybe_conn_cell.borrow_mut();
                    if maybe_conn.is_none() {
                        match db_pool.get() {
                            Ok(conn) => *maybe_conn = Some(conn),
                            Err(e) => {
                                let e: Error = ectx!(err e, ErrorSource::R2D2, ErrorKind::Internal);
                                return Err(e.into());
                            }
                        }
                    }
                }
                f()
            })
        }))
    }

    fn execute_transaction<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Fail,
    {
        let db_pool = self.db_pool.clone();
        Box::new(self.db_thread_pool.spawn_fn(move || {
            DB_CONN.with(move |maybe_conn_cell| -> Result<T, E> {
                {
                    let mut maybe_conn = maybe_conn_cell.borrow_mut();
                    if maybe_conn.is_none() {
                        match db_pool.get() {
                            Ok(conn) => *maybe_conn = Some(conn),
                            Err(e) => {
                                let e: Error = ectx!(err e, ErrorSource::R2D2, ErrorKind::Internal);
                                return Err(e.into());
                            }
                        }
                    }
                }
                with_tls_connection(move |conn| {
                    conn.transaction::<_, FailureError, _>(|| f().map_err(From::from))
                        .map_err(ectx!(ErrorSource::Transaction, ErrorKind::Internal))
                }).map_err(|e| e.into())
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
