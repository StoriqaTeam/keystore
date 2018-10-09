use super::error::*;
use ethcore_transaction::{Transaction, Action};
use ethereum_types::U256;
use models::*;
use prelude::*;

pub trait BlockchainSigner: Send + Sync + 'static {
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error>;
}

#[derive(Default)]
pub struct BlockchainSignerImpl {
    stq_gas_limit: usize,
    stq_contract_address: String,
}

impl BlockchainSignerImpl {
    fn new(stq_gas_limit: usize, stq_contract_address: String) -> Self {
        Self {
            stq_gas_limit,
            stq_contract_address,
        }
    }
}

impl BlockchainSigner for BlockchainSignerImpl {
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error> {
        let UnsignedTransaction {
            from,
            to,
            currency,
            value,
            fee_price,
            nonce: maybe_nonce,
            ..
        } = tx;
        let nonce = maybe_nonce.ok_or(ectx!(err ErrorKind::MissingNonce, ErrorSource::Signer, ErrorKind::MissingNonce))?;
        let nonce: U256 = nonce.into();
        let gas_price: U256 = fee_price.into();
        let gas: U256 = self.stq_gas_limit.into();
        let value: U256 = value.into();
        let to = to.to_h160()?;
        let action = Action::Call(to);

        let tx = Transaction {
            nonce,
            gas_price,
            gas,
            action,
            value,
        }
        unimplemented!()
    }
}
