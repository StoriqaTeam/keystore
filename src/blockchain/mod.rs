mod bitcoin;
mod error;
mod ethereum;
#[cfg(test)]
mod mocks;
mod utils;

pub use self::error::*;
#[cfg(test)]
pub use self::mocks::*;

use self::bitcoin::BitcoinService;
use self::ethereum::EthereumService;
use config::BtcNetwork;

use models::*;

pub trait BlockchainService: Send + Sync + 'static {
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error>;
    fn approve(&self, key: PrivateKey, tx: ApproveInput) -> Result<RawTransaction, Error>;
    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error>;
    fn derive_address(&self, currency: Currency, key: PrivateKey) -> Result<BlockchainAddress, Error>;
}

pub struct BlockchainServiceImpl {
    ethereum_service: EthereumService,
    bitcoin_service: BitcoinService,
}

impl BlockchainServiceImpl {
    pub fn new(
        stq_gas_limit: usize,
        stq_contract_address: String,
        stq_transfer_from_method_number: String,
        stq_approve_method_number: String,
        chain_id: Option<u64>,
        btc_network: BtcNetwork,
    ) -> Self {
        let ethereum_service = EthereumService::new(
            stq_gas_limit,
            stq_contract_address,
            stq_transfer_from_method_number,
            stq_approve_method_number,
            chain_id,
        );
        let bitcoin_service = BitcoinService::new(btc_network);
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
    fn approve(&self, key: PrivateKey, input: ApproveInput) -> Result<RawTransaction, Error> {
        self.ethereum_service.approve(key, input)
    }
    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error> {
        match currency {
            Currency::Eth | Currency::Stq => self.ethereum_service.generate_key(currency),
            Currency::Btc => self.bitcoin_service.generate_key(currency),
        }
    }

    fn derive_address(&self, currency: Currency, key: PrivateKey) -> Result<BlockchainAddress, Error> {
        match currency {
            Currency::Eth | Currency::Stq => self.ethereum_service.derive_address(currency, key),
            Currency::Btc => self.bitcoin_service.derive_address(currency, key),
        }
    }
}
