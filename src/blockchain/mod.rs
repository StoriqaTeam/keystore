mod bitcoin;
mod error;
mod ethereum;
#[cfg(test)]
mod mocks;

pub use self::error::*;
#[cfg(test)]
pub use self::mocks::*;

use self::bitcoin::BitcoinService;
use self::ethereum::EthereumService;

use models::*;

pub trait BlockchainService: Send + Sync + 'static {
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error>;
    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error>;
}

#[derive(Default)]
pub struct BlockchainServiceImpl {
    ethereum_service: EthereumService,
    bitcoin_service: BitcoinService,
}

impl BlockchainServiceImpl {
    pub fn new(stq_gas_limit: usize, stq_contract_address: String, stq_transfer_method_number: String, chain_id: Option<u64>) -> Self {
        let ethereum_service = EthereumService::new(stq_gas_limit, stq_contract_address, stq_transfer_method_number, chain_id);
        let bitcoin_service = BitcoinService::new();
        Self {
            ethereum_service,
            bitcoin_service,
        }
    }
}

impl BlockchainService for BlockchainServiceImpl {
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error> {
        match tx.currency {
            Currency::Eth | Currency::Stq => self.ethereum_service.sign(key, tx),
            Currency::Btc => self.bitcoin_service.sign(key, tx),
        }
    }
    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error> {
        match currency {
            Currency::Eth | Currency::Stq => self.ethereum_service.generate_key(currency),
            Currency::Btc => self.bitcoin_service.generate_key(currency),
        }
    }
}
