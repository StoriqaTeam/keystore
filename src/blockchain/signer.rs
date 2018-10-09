use super::error::Error;
use ethcore_transaction::Transaction;
use models::*;

pub trait BlockchainSigner: Send + Sync + 'static {
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error>;
}

#[derive(Default)]
pub struct BlockchainSignerImpl;

impl BlockchainSigner for BlockchainSignerImpl {
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error> {
        unimplemented!()
    }
}
