use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use models::*;
use prelude::*;
use schema::keys::dsl::*;

pub trait KeysRepo: Send + Sync + 'static {
    fn list(&self, current_user_id: UserId, offset: i64, limit: i64) -> Result<Vec<Key>, Error>;
    fn create(&self, payload: NewKey) -> Result<Key, Error>;
    // We don't check currency, since there's case when you want to transfer
    // ether to stq account (to be able to make withdrawal)
    fn find_by_address(&self, current_user_id: UserId, address: BlockchainAddress) -> Result<Option<Key>, Error>;
}

pub struct KeysRepoImpl {
    aes_key: String,
}

impl KeysRepoImpl {
    pub fn new(aes_key: String) -> Self {
        KeysRepoImpl { aes_key }
    }
}

impl KeysRepo for KeysRepoImpl {
    fn list(&self, current_user_id: UserId, offset: i64, limit: i64) -> Result<Vec<Key>, Error> {
        with_tls_connection(|conn| {
            let encrypted_keys = keys
                .filter(owner_id.eq(current_user_id))
                .offset(offset)
                .limit(limit)
                .get_results::<EncryptedKey>(conn)
                .map_err(ectx!(try ErrorKind::Internal))?;
            Ok(encrypted_keys
                .into_iter()
                .map(|encrypted_key| Key::from_encrypted(encrypted_key, &self.aes_key))
                .collect())
        })
    }

    fn find_by_address(&self, current_user_id: UserId, address: BlockchainAddress) -> Result<Option<Key>, Error> {
        with_tls_connection(|conn| {
            let maybe_enc_key = keys
                .filter(owner_id.eq(current_user_id))
                .filter(blockchain_address.eq(address))
                .limit(1)
                .get_results(conn)
                .map(|ks| ks.get(0).cloned())
                .map_err(ectx!(try ErrorKind::Internal))?;
            Ok(maybe_enc_key.map(|enc_key| Key::from_encrypted(enc_key, &self.aes_key)))
        })
    }

    fn create(&self, payload: NewKey) -> Result<Key, Error> {
        let payload = NewEncryptedKey::from_new_key(payload, &self.aes_key);
        let payload_clone = payload.clone();
        with_tls_connection(move |conn| {
            diesel::insert_into(keys)
                .values(payload.clone())
                .get_result::<EncryptedKey>(conn)
                .map(|encrypted_key| Key::from_encrypted(encrypted_key, &self.aes_key))
                .map_err(move |e| {
                    let kind = ErrorKind::from_diesel(&e);
                    ectx!(err e, kind => payload_clone)
                })
        })
    }
}
