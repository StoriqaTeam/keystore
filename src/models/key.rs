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
#[derive(FromSqlRow, AsExpression, Clone, PartialEq, Eq)]
#[sql_type = "VarChar"]
pub struct PrivateKey(String);
derive_newtype_sql!(private_key_id, VarChar, PrivateKey, PrivateKey);
mask_logs!(PrivateKey);

impl PrivateKey {
    pub fn new(data: String) -> Self {
        PrivateKey(data)
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn encrypt(&self, main_key: &str) -> String {
        let key = decode_hex(main_key);
        let mut random = rand::OsRng::new().unwrap();
        let mut iv = [0u8; 16];
        random.fill_bytes(&mut iv);
        let decrypted = decode_hex(&self.0);
        println!("dec: {:?}, key: {:?}, iv: {:?}", &decrypted, &key, &iv);
        let encrypted = aes_enc(&decrypted, &key, &iv).unwrap();
        let mut res = encode_hex(&iv);
        res.push_str(&encode_hex(&encrypted));
        res
    }

    pub fn decrypt(encrypted: &str, main_key: &str) -> Self {
        let key = decode_hex(main_key);
        let iv = decode_hex(&encrypted[0..32]);
        let encrypted = decode_hex(&encrypted[32..]);
        let decrypted = aes_dec(&encrypted, &key, &iv).unwrap();
        PrivateKey(encode_hex(&decrypted))
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

#[derive(Debug, Queryable, Clone)]
pub struct Key {
    pub id: KeyId,
    pub private_key: PrivateKey,
    pub blockchain_address: BlockchainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "keys"]
pub struct NewKey {
    pub id: KeyId,
    pub private_key: PrivateKey,
    pub blockchain_address: BlockchainAddress,
    pub currency: Currency,
    pub owner_id: UserId,
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
            let mut main_key = [0u8; 32];
            random.fill_bytes(&mut main_key);
            let main_key = encode_hex(&main_key);
            assert_eq!(pk, PrivateKey::decrypt(&pk.encrypt(&main_key), &main_key));
        }
    }
}
