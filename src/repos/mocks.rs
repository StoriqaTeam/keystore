use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::error::*;
use super::executor::DbExecutor;
use super::keys::*;
use super::users::*;
use models::*;
use prelude::*;

#[derive(Clone)]
pub struct KeysRepoMock {
    data: Arc<Mutex<Vec<Key>>>,
}

impl KeysRepoMock {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl KeysRepo for KeysRepoMock {
    fn list(&self, current_user_id: UserId, offset: i64, limit: i64) -> Result<Vec<Key>, Error> {
        let data = self.data.lock().unwrap();
        Ok(data
            .iter()
            .filter(|x| x.owner_id == current_user_id)
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect())
    }
    fn create(&self, payload: NewKey) -> Result<Key, Error> {
        let mut data = self.data.lock().unwrap();
        let key = Key {
            id: payload.id,
            currency: payload.currency,
            blockchain_address: payload.blockchain_address,
            owner_id: payload.owner_id,
            private_key: payload.private_key,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };
        data.push(key.clone());
        Ok(key)
    }
}

#[derive(Clone)]
pub struct UsersRepoMock {
    data: Arc<Mutex<Vec<User>>>,
}

impl UsersRepoMock {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl UsersRepo for UsersRepoMock {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<User>, Error> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().filter(|x| x.authentication_token == token).nth(0).cloned())
    }

    fn create(&self, payload: NewUser) -> Result<User, Error> {
        let mut data = self.data.lock().unwrap();
        let res = User {
            id: payload.id,
            name: payload.name,
            authentication_token: payload.authentication_token,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };
        data.push(res.clone());
        Ok(res)
    }
}

#[derive(Clone)]
pub struct DbExecutorMock;

impl DbExecutorMock {
    pub fn new() -> Self {
        DbExecutorMock
    }
}

impl DbExecutor for DbExecutorMock {
    fn execute<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Send + 'static,
    {
        Box::new(f().into_future())
    }
    fn execute_transaction<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Send + 'static,
    {
        Box::new(f().into_future())
    }

    #[cfg(test)]
    fn execute_test_transaction<F, T, E>(&self, f: F) -> Box<Future<Item = T, Error = E> + Send + 'static>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: From<Error> + Send + 'static,
    {
        Box::new(f().into_future())
    }
}
