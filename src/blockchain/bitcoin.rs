use super::error::*;
use super::BlockchainService;
use models::*;

#[derive(Default)]
pub struct BitcoinService;

impl BitcoinService {
    pub fn new() -> Self {
        BitcoinService
    }
}

impl BlockchainService for BitcoinService {
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error> {
        unimplemented!()
    }

    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error> {
        unimplemented!()
    }
}
