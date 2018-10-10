use super::error::*;
use ethcore_transaction::{Action, Transaction};
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
    stq_transfer_method_number: String,
}

impl BlockchainSignerImpl {
    fn new(stq_gas_limit: usize, stq_contract_address: String, stq_transfer_method_number: String) -> Self {
        Self {
            stq_gas_limit,
            stq_contract_address,
            stq_transfer_method_number,
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
        let nonce = maybe_nonce.ok_or(ectx!(try err ErrorKind::MissingNonce, ErrorSource::Signer, ErrorKind::MissingNonce))?;
        let nonce: U256 = nonce.into();
        let gas_price: U256 = fee_price.into();
        let gas: U256 = self.stq_gas_limit.into();
        let value: U256 = match currency {
            Currency::Eth => value.into(),
            Currency::Stq => 0.into(),
        };
        let to = to.to_h160()?;
        let action = Action::Call(to);
        // let data = match currency {
        //     Currency::Eth => Vec::new(),
        //     Currency::Stq => Vec::new(),
        // };

        // let tx = Transaction {
        //     nonce,
        //     gas_price,
        //     gas,
        //     action,
        //     value,
        // };
        unimplemented!()
    }
}

fn serialize_amount(amount: Amount) -> Vec<u8> {
    to_padded_32_bytes(&amount.to_bytes())
}

fn serialize_address(address: BlockchainAddress) -> Result<Vec<u8>, Error> {
    let chars: Vec<char> = address.clone().into_inner().chars().collect();
    let res: Result<Vec<u8>, Error> = chars
        .chunks(2)
        .map(|chunk| {
            let address = address.clone();
            if chunk.len() < 2 {
                let e: Error = ErrorKind::MalformedAddress.into();
                return Err(ectx!(err e, ErrorContext::AddressConvert, ErrorKind::MalformedAddress => address));
            }
            let string = format!("{}{}", chunk[0], chunk[1]);
            u8::from_str_radix(&string, 16).map_err(ectx!(ErrorContext::AddressConvert, ErrorKind::MalformedAddress => address))
        }).collect();
    res.map(|data| to_padded_32_bytes(&data))
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

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut res = String::with_capacity(bytes.len() * 2);
    for byte in bytes.iter() {
        res.push_str(&format!("{:02x}", byte));
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::*;

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
