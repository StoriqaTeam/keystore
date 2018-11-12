use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use super::error::*;
use super::BlockchainService;
use models::*;

pub struct BlockchainServiceMock;

impl BlockchainService for BlockchainServiceMock {
    fn generate_key(&self, _currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error> {
        let key: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
        let address: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        Ok((PrivateKey::new(key), BlockchainAddress::new(address)))
    }

    fn sign(&self, _key: PrivateKey, _tx: UnsignedTransaction) -> Result<RawTransaction, Error> {
        let tx: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
        Ok(RawTransaction::new(tx))
    }

    fn approve(&self, key: PrivateKey, tx: ApproveInput) -> Result<RawTransaction, Error> {
        let tx: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
        Ok(RawTransaction::new(tx))
    }
}
