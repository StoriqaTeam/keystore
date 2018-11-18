use std::time::SystemTime;

use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use rand::RngCore;
use std::fmt::{self, Debug, Display};
use uuid::Uuid;

use super::currency::Currency;
use super::user::UserId;
use schema::keys;
use utils::{decode_hex, decrypt as aes_dec, encode_hex, encrypt as aes_enc};

#[derive(Serialize, Deserialize, PartialEq, Eq, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct KeyId(Uuid);
derive_newtype_sql!(key_id, SqlUuid, KeyId, KeyId);

impl Default for KeyId {
    fn default() -> Self {
        KeyId(Uuid::new_v4())
    }
}

impl Debug for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

/// Hex encoded private key
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateKey(String);
mask_logs!(PrivateKey);

/// Hex encoded encrypted private key
#[derive(FromSqlRow, AsExpression, Clone, PartialEq, Eq)]
#[sql_type = "VarChar"]
pub struct EncryptedPrivateKey(String);
derive_newtype_sql!(encrypted_private_key, VarChar, EncryptedPrivateKey, EncryptedPrivateKey);
mask_logs!(EncryptedPrivateKey);

impl PrivateKey {
    pub fn new(data: String) -> Self {
        PrivateKey(data)
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn from_encrypted(encrypted_pk: EncryptedPrivateKey, aes_key: &str) -> Self {
        let key = decode_hex(aes_key);
        let encrypted = encrypted_pk.into_inner();
        let iv = decode_hex(&encrypted[0..32]);
        let encrypted = decode_hex(&encrypted[32..]);
        let decrypted = aes_dec(&encrypted, &key, &iv).unwrap();
        PrivateKey(encode_hex(&decrypted))
    }
}

impl EncryptedPrivateKey {
    pub fn new(data: String) -> Self {
        EncryptedPrivateKey(data)
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn from_private_key(pk: PrivateKey, aes_key: &str) -> Self {
        let key = decode_hex(aes_key);
        let mut random = rand::OsRng::new().unwrap();
        let mut iv = [0u8; 16];
        random.fill_bytes(&mut iv);
        let decrypted = decode_hex(&pk.into_inner());
        let encrypted = aes_enc(&decrypted, &key, &iv).unwrap();
        let mut res = encode_hex(&iv);
        res.push_str(&encode_hex(&encrypted));
        EncryptedPrivateKey(res)
    }
}

/// Hex encoded blockchain address
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct BlockchainAddress(String);
derive_newtype_sql!(blockchain_address, VarChar, BlockchainAddress, BlockchainAddress);

impl BlockchainAddress {
    pub fn new(data: String) -> Self {
        BlockchainAddress(data)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Key {
    pub id: KeyId,
    pub private_key: PrivateKey,
    pub blockchain_address: BlockchainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Key {
    pub fn from_encrypted(encrypted_key: EncryptedKey, aes_key: &str) -> Self {
        Key {
            id: encrypted_key.id,
            private_key: PrivateKey::from_encrypted(encrypted_key.private_key, &aes_key),
            blockchain_address: encrypted_key.blockchain_address,
            currency: encrypted_key.currency,
            owner_id: encrypted_key.owner_id,
            created_at: encrypted_key.created_at,
            updated_at: encrypted_key.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewKey {
    pub id: KeyId,
    pub private_key: PrivateKey,
    pub blockchain_address: BlockchainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
}

#[derive(Debug, Queryable, Clone)]
pub struct EncryptedKey {
    pub id: KeyId,
    pub private_key: EncryptedPrivateKey,
    pub blockchain_address: BlockchainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "keys"]
pub struct NewEncryptedKey {
    pub id: KeyId,
    pub private_key: EncryptedPrivateKey,
    pub blockchain_address: BlockchainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
}

impl NewEncryptedKey {
    pub fn from_new_key(new_key: NewKey, aes_key: &str) -> Self {
        NewEncryptedKey {
            id: new_key.id,
            private_key: EncryptedPrivateKey::from_private_key(new_key.private_key, &aes_key),
            blockchain_address: new_key.blockchain_address,
            currency: new_key.currency,
            owner_id: new_key.owner_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use utils::encode_hex;
    #[test]
    fn test_encrypt() {
        let mut random = rand::OsRng::new().unwrap();
        for _ in 0..1000 {
            let number_of_elems: u8 = random.gen();
            let mut pk = Vec::with_capacity(number_of_elems as usize);
            pk.resize(number_of_elems as usize, 0);
            random.fill_bytes(&mut pk);
            let pk = PrivateKey(encode_hex(&pk));
            let mut aes_key = [0u8; 32];
            random.fill_bytes(&mut aes_key);
            let aes_key = encode_hex(&aes_key);
            assert_eq!(
                pk.clone(),
                PrivateKey::from_encrypted(EncryptedPrivateKey::from_private_key(pk, &aes_key), &aes_key)
            );
        }
    }
}
