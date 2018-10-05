use models::*;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub trait KeyGenerator: Send + Sync + 'static {
    fn generate_key(&self, currency: Currency) -> (PrivateKey, BlockChainAddress);
}

pub struct KeyGeneratorImpl;

impl KeyGenerator for KeyGeneratorImpl {
    fn generate_key(&self, currency: Currency) -> (PrivateKey, BlockChainAddress) {
        let key: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
        let address: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        (PrivateKey::new(key), BlockChainAddress::new(address))
    }
}
