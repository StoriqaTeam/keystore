use rlp;
use std::str::FromStr;

use super::error::*;
use super::utils::{bytes_to_hex, hex_to_bytes};
use super::BlockchainService;
use ethcore_transaction::{Action, Transaction};
use ethereum_types::{H160, U256};
use ethkey::{Generator, Random};
use ethkey::{KeyPair, Secret};
use failure::err_msg;
use models::*;
use prelude::*;

#[derive(Default)]
pub struct EthereumService {
    stq_gas_limit: usize,
    eth_gas_limit: usize,
    stq_contract_address: String,
    stq_transfer_from_method_number: String,
    stq_approve_method_number: String,
    chain_id: Option<u64>,
}

impl EthereumService {
    pub fn new(
        stq_gas_limit: usize,
        eth_gas_limit: usize,
        stq_contract_address: String,
        stq_transfer_from_method_number: String,
        stq_approve_method_number: String,
        chain_id: Option<u64>,
    ) -> Self {
        EthereumService {
            stq_gas_limit,
            eth_gas_limit,
            stq_contract_address,
            stq_transfer_from_method_number,
            stq_approve_method_number,
            chain_id,
        }
    }
}

impl BlockchainService for EthereumService {
    fn derive_address(&self, _currency: Currency, key: PrivateKey) -> Result<BlockchainAddress, Error> {
        let secret = private_key_to_secret(key.clone())?;
        Ok(BlockchainAddress::new(format!(
            "{:x}",
            KeyPair::from_secret(secret)
                .map_err({
                    let error = ValidationError::MalformedPrivateKey {
                        value: key.clone().into_inner(),
                    };
                    ectx!(try ErrorKind::InvalidPrivateKey(error))
                })?
                .address()
        )))
    }
    fn generate_key(&self, _currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error> {
        let mut random = Random;
        let pair = random.generate().map_err(ectx!(try ErrorSource::Random, ErrorKind::Internal))?;
        let private_key = PrivateKey::new(format!("{:x}", pair.secret()));
        let blockchain_address = BlockchainAddress::new(format!("{:x}", pair.address()));
        Ok((private_key, blockchain_address))
    }
    fn approve(&self, key: PrivateKey, input: ApproveInput) -> Result<RawTransaction, Error> {
        let ApproveInput {
            approve_address,
            value,
            fee_price,
            nonce,
            ..
        } = input;
        let nonce: U256 = nonce.into();
        // this is generally a big number, so ok to cast to int
        let gas_price: U256 = Amount::new(fee_price as u128).into();
        let gas: U256 = self.stq_gas_limit.into();
        let tx_value: U256 = 0.into();
        let to = H160::from_str(&self.stq_contract_address)
            .map_err(ectx!(try ErrorContext::MalformedStqContractAddress, ErrorKind::Internal => self.stq_contract_address))?;
        let action = Action::Call(to);
        let mut data: Vec<u8> = Vec::new();
        let method = hex_to_bytes(self.stq_approve_method_number.clone())
            .map_err(ectx!(try ErrorContext::MalformedMethodNumber, ErrorKind::Internal => self.stq_approve_method_number))?;
        let approve_address =
            serialize_address(approve_address.clone()).map_err(ectx!(try ErrorSource::Serde, ErrorKind::Internal => approve_address))?;
        let value = serialize_amount(value);
        data.extend(method.iter());
        data.extend(approve_address.iter());
        data.extend(value.iter());

        let tx = Transaction {
            nonce,
            gas_price,
            gas,
            action,
            value: tx_value,
            data,
        };
        let secret = private_key_to_secret(key)?;
        let signed = tx.sign(&secret, self.chain_id);
        let raw_data = rlp::encode(&signed).to_vec();
        let raw_hex_data = bytes_to_hex(&raw_data);
        Ok(RawTransaction::new(raw_hex_data))
    }
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
        let nonce = maybe_nonce.ok_or(ErrorKind::InvalidUnsignedTransaction(ValidationError::MissingNonce))?;
        let nonce: U256 = nonce.into();
        let gas_price: U256 = Amount::new(fee_price as u128).into();
        let gas: U256 = match currency {
            Currency::Eth => self.eth_gas_limit.into(),
            Currency::Stq => self.stq_gas_limit.into(),
            other => {
                let cause = err_msg("attempted to sign non-ethereum currency with ethereum algos");
                let error = ValidationError::UnsupportedCurrency { value: other.to_string() };
                Err(ectx!(try err cause, ErrorKind::InvalidUnsignedTransaction(error)))?
            }
        };
        let tx_value: U256 = match currency {
            Currency::Eth => value.into(),
            Currency::Stq => 0.into(),
            other => {
                let cause = err_msg("attempted to sign non-ethereum currency with ethereum algos");
                let error = ValidationError::UnsupportedCurrency { value: other.to_string() };
                Err(ectx!(try err cause, ErrorKind::InvalidUnsignedTransaction(error)))?
            }
        };
        let action = match currency {
            Currency::Eth => {
                let to = H160::from_str(&to.clone().into_inner()).map_err({
                    let error = ValidationError::MalformedHexString {
                        value: to.clone().into_inner(),
                    };
                    ectx!(try ErrorKind::InvalidUnsignedTransaction(error))
                })?;
                Action::Call(to)
            }
            Currency::Stq => {
                let to = H160::from_str(&self.stq_contract_address).map_err(
                    ectx!(try ErrorContext::MalformedStqContractAddress, ErrorKind::Internal => self.stq_contract_address.clone()),
                )?;
                Action::Call(to)
            }
            other => {
                let cause = err_msg("attempted to sign non-ethereum currency with ethereum algos");
                let error = ValidationError::UnsupportedCurrency { value: other.to_string() };
                Err(ectx!(try err cause, ErrorKind::InvalidUnsignedTransaction(error)))?
            }
        };
        let data = match currency {
            Currency::Eth => Vec::new(),
            Currency::Stq => {
                let mut data: Vec<u8> = Vec::new();
                let method = hex_to_bytes(self.stq_transfer_from_method_number.clone()).map_err({
                    ectx!(try ErrorContext::MalformedMethodNumber, ErrorKind::Internal => self.stq_transfer_from_method_number.clone())
                })?;
                let from = serialize_address(from.clone()).map_err({
                    let error = ValidationError::MalformedAddress { value: from.into_inner() };
                    ectx!(try ErrorKind::InvalidUnsignedTransaction(error))
                })?;
                let to = serialize_address(to.clone()).map_err({
                    let error = ValidationError::MalformedAddress { value: to.into_inner() };
                    ectx!(try ErrorKind::InvalidUnsignedTransaction(error))
                })?;
                let value = serialize_amount(value);
                data.extend(method.iter());
                data.extend(from.iter());
                data.extend(to.iter());
                data.extend(value.iter());
                data
            }
            other => {
                let cause = err_msg("attempted to sign non-ethereum currency with ethereum algos");
                let error = ValidationError::UnsupportedCurrency { value: other.to_string() };
                Err(ectx!(try err cause, ErrorKind::InvalidUnsignedTransaction(error)))?
            }
        };

        let tx = Transaction {
            nonce,
            gas_price,
            gas,
            action,
            value: tx_value,
            data,
        };
        let secret = private_key_to_secret(key)?;
        let signed = tx.sign(&secret, self.chain_id);
        let raw_data = rlp::encode(&signed).to_vec();
        let raw_hex_data = bytes_to_hex(&raw_data);
        Ok(RawTransaction::new(raw_hex_data))
    }
}

fn private_key_to_secret(key: PrivateKey) -> Result<Secret, Error> {
    let hex_pk = key.clone().into_inner();
    let bytes = hex_to_bytes(hex_pk.clone()).map_err({
        let error = ValidationError::MalformedPrivateKey { value: hex_pk.clone() };
        ectx!(try ErrorKind::InvalidPrivateKey(error))
    })?;
    Secret::from_slice(&bytes).ok_or({
        let error = ValidationError::MalformedPrivateKey { value: hex_pk };
        ErrorKind::InvalidPrivateKey(error).into()
    })
}

fn serialize_amount(amount: Amount) -> Vec<u8> {
    to_padded_32_bytes(&amount.bytes())
}

fn serialize_address(address: BlockchainAddress) -> Result<Vec<u8>, Error> {
    hex_to_bytes(address.into_inner()).map(|data| to_padded_32_bytes(&data))
}

fn to_padded_32_bytes(data: &[u8]) -> Vec<u8> {
    let zeros_len = 32 - data.len();
    let mut res = Vec::with_capacity(32);
    for _ in 0..zeros_len {
        res.push(0);
    }
    res.extend(data.iter());
    res
}

#[cfg(test)]
mod tests {
    use super::super::BlockchainService;
    use super::*;

    #[test]
    fn test_sign() {
        let ethereum_service = EthereumService {
            stq_gas_limit: 100000,
            eth_gas_limit: 21000,
            stq_contract_address: "1bf2092a42166b2ae19b7b23752e7d2dab5ba91a".to_string(),
            stq_transfer_from_method_number: "23b872dd".to_string(),
            stq_approve_method_number: "095ea7b3".to_string(),
            chain_id: Some(42),
        };
        let private_key = PrivateKey::new("b3c0e85a511cc6d21423a386de29dcf2cda6b2f2fa5ebb47948401bbb90458db".to_string());
        let to = BlockchainAddress::new("00d44DD2f6a2d2005326Db58eC5137204C5Cba5A".to_string());
        // from is inferred from private_key for eth, but for stq we use transferFrom => it's needed
        let from = BlockchainAddress::new("B3683B4DE1fc502807464B55d151e8e2D2c19cb5".to_string());
        let cases = [
            (
                UnsignedTransaction {
                    id: TransactionId::default(),
                    from: from.clone(),
                    to: to.clone(),
                    currency: Currency::Eth,
                    value: Amount::new(25000000000000000000),
                    fee_price: 30000000000.0f64,
                    nonce: Some(0),
                    utxos: None,
                },
                "f86e808506fc23ac00830186a09400d44dd2f6a2d2005326db58ec5137204c5cba5a89015af1d78b58c400008077a09bb23536f025bc054d87c68faf2dcb99141a0be6ab28ea888974d4a9b5d9473ca0436070757106922b3c65c81592d5c8ea55fac876b78b8c5ce946711ff8c74cb4",
            ),
            (
                UnsignedTransaction {
                    id: TransactionId::default(),
                    from: from.clone(),
                    to: to.clone(),
                    currency: Currency::Stq,
                    value: Amount::new(25_000_000_000_000_000_000),
                    fee_price: 30000000000.0f64,
                    nonce: Some(0),
                    utxos: None,
                },
                "f8ca808506fc23ac00830186a0941bf2092a42166b2ae19b7b23752e7d2dab5ba91a80b86423b872dd000000000000000000000000b3683b4de1fc502807464b55d151e8e2d2c19cb500000000000000000000000000d44dd2f6a2d2005326db58ec5137204c5cba5a0000000000000000000000000000000000000000000000015af1d78b58c4000078a0beba6b3493ea0a04c8fc45b4c85e44bbb6367cc4a96b200b35507ab80e8d5b03a03b164cd3c5a235b280b8fc47be3e58b14ee67992ada8fc64d6773175eda0f1b8",
            ),
        ];
        for case in cases.into_iter() {
            let (input, expected) = case.clone();
            let output = ethereum_service.sign(private_key.clone(), input).unwrap();
            assert_eq!(output, RawTransaction::new(expected.to_string()));
        }
    }

    #[test]
    fn test_approve() {
        let ethereum_service = EthereumService {
            stq_gas_limit: 100000,
            eth_gas_limit: 21000,
            stq_contract_address: "1bf2092a42166b2ae19b7b23752e7d2dab5ba91a".to_string(),
            stq_transfer_from_method_number: "23b872dd".to_string(),
            stq_approve_method_number: "095ea7b3".to_string(),
            chain_id: Some(42),
        };
        let private_key = PrivateKey::new("b3c0e85a511cc6d21423a386de29dcf2cda6b2f2fa5ebb47948401bbb90458db".to_string());
        let approve_address = BlockchainAddress::new("00d44DD2f6a2d2005326Db58eC5137204C5Cba5A".to_string());
        // this one is ignored by approve function
        let address = BlockchainAddress::new("B3683B4DE1fc502807464B55d151e8e2D2c19cb5".to_string());
        let cases = [
            (
                ApproveInput {
                    id: TransactionId::default(),
                    address,
                    approve_address,
                    currency: Currency::Stq,
                    value: Amount::new(25000000000000000000),
                    fee_price: 30000000000.0f64,
                    nonce: 0,
                },
                "f8aa808506fc23ac00830186a0941bf2092a42166b2ae19b7b23752e7d2dab5ba91a80b844095ea7b300000000000000000000000000d44dd2f6a2d2005326db58ec5137204c5cba5a0000000000000000000000000000000000000000000000015af1d78b58c4000077a066cc102349d86e0b09b1d8ea7cdd4f61183ef2bc9bc3ba7f46e602a2f017af7fa061e1d644965908efea05e4dc609b9c434b723a27ebbde4a2c71a50e4705490aa",
            ),
        ];
        for case in cases.into_iter() {
            let (input, expected) = case.clone();
            let output = ethereum_service.approve(private_key.clone(), input).unwrap();
            assert_eq!(output, RawTransaction::new(expected.to_string()));
        }
    }

    #[test]
    fn test_serialize_address() {
        let cases = [
            (
                "8A54941dB68A89d63Af5064F53B1C8Fc832B4D89",
                "0000000000000000000000008a54941db68a89d63af5064f53b1c8fc832b4d89",
            ),
            (
                "0054941dB68A89d63Af5064F53B1C8Fc832B4D89",
                "0000000000000000000000000054941db68a89d63af5064f53b1c8fc832b4d89",
            ),
            (
                "0054941dB68A89d63Af5064F53B1C8Fc83010089",
                "0000000000000000000000000054941db68a89d63af5064f53b1c8fc83010089",
            ),
        ];
        for case in cases.into_iter() {
            let (input, expected) = case.clone();
            let address = BlockchainAddress::new(input.to_string());
            let serialized = serialize_address(address).unwrap();
            assert_eq!(bytes_to_hex(&serialized), expected);
        }
    }

    #[test]
    fn test_serialize_amount() {
        let cases = [(180000000000u128, "00000000000000000000000000000000000000000000000000000029e8d60800")];
        for case in cases.into_iter() {
            let (input, expected) = case.clone();
            let amount = Amount::new(input);
            let serialized = serialize_amount(amount);
            assert_eq!(bytes_to_hex(&serialized), expected);
        }
    }

}
