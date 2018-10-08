mod error;
#[cfg(test)]
mod mocks;

pub use self::error::*;
#[cfg(test)]
pub use self::mocks::*;

use ethkey::{Generator, Random};
use models::*;
use prelude::*;

pub trait KeyGenerator: Send + Sync + 'static {
    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error>;
}

pub struct KeyGeneratorImpl;

impl KeyGenerator for KeyGeneratorImpl {
    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error> {
        match currency {
            Currency::Eth => self.generate_ethereum_key(),
            Currency::Stq => self.generate_ethereum_key(),
        }
    }
}

impl KeyGeneratorImpl {
    fn generate_ethereum_key(&self) -> Result<(PrivateKey, BlockchainAddress), Error> {
        let mut random = Random;
        let pair = random.generate().map_err(ectx!(try ErrorSource::Random, ErrorKind::Internal))?;
        let private_key = PrivateKey::new(format!("{:x}", pair.secret()));
        let blockchain_address = BlockchainAddress::new(format!("{:x}", pair.address()));
        Ok((private_key, blockchain_address))
    }
}
