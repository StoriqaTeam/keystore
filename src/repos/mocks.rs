use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::error::*;
use super::keys::*;
use super::users::*;
use models::*;

#[derive(Clone)]
struct KeysRepoMock {
    data: Arc<Mutex<Vec<Key>>>,
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
struct UsersRepoMock {
    data: Arc<Mutex<Vec<User>>>,
}

impl UsersRepo for UsersRepoMock {
    fn find_user_by_authentication_token(&self, token: AuthenticationToken) -> Result<Option<User>, Error> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().filter(|x| x.authentication_token == token).nth(1).cloned())
    }
}
