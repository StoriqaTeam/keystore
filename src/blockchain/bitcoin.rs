use btcchain::{OutPoint, Transaction, TransactionInput, TransactionOutput};
use btccrypto::sha256;
use btckey::generator::{Generator, Random};
use btckey::{Address, DisplayLayout, Error as BtcKeyError, KeyPair, Network, Private as BtcPrivateKey, Type as AddressType};
use btcprimitives::hash::H256;
use btcscript::Builder as ScriptBuilder;
use btcserialization::serialize;
use config::BtcNetwork;

use super::error::*;
use super::utils::{bytes_to_hex, hex_to_bytes};
use super::BlockchainService;
use models::*;
use prelude::*;

pub struct BitcoinService {
    btc_network: BtcNetwork,
}

impl BitcoinService {
    // https://en.bitcoin.it/wiki/OP_CHECKSIG
    // https://bitcoin.stackexchange.com/questions/3374/how-to-redeem-a-basic-tx
    fn sign_with_options(
        &self,
        key: PrivateKey,
        input_tx: UnsignedTransaction,
        rbf: bool,
        lock_time: Option<u32>,
    ) -> Result<RawTransaction, Error> {
        let input_value = input_tx
            .value
            .u64()
            .ok_or(ectx!(try err ErrorKind::Overflow, ErrorKind::Overflow))?;
        let utxos = self.needed_utxos(&input_tx.utxos.clone().unwrap_or_default(), input_tx.value)?;

        let from_address = input_tx.from.clone().into_inner();
        let address_from: Address = from_address.parse().map_err(|e: BtcKeyError| {
            let e = format_err!("{}", e);
            ectx!(try err e, ErrorKind::MalformedAddress)
        })?;
        if address_from.kind != AddressType::P2PKH {
            return Err(ectx!(err ErrorContext::UnsupportedAddress, ErrorKind::MalformedAddress => input_tx));
        }
        let address_from_hash = address_from.hash;
        let script_sig = ScriptBuilder::build_p2pkh(&address_from_hash);

        let inputs: Result<Vec<TransactionInput>, Error> = utxos
            .iter()
            .map(|utxo| -> Result<TransactionInput, Error> {
                let Utxo { tx_hash, index, .. } = utxo;
                let tx_hash: H256 = tx_hash
                    .parse()
                    .map_err(|_| ectx!(try err ErrorKind::MalformedHexString, ErrorKind::MalformedHexString))?;
                let tx_hash = tx_hash.reversed();
                let outpoint = OutPoint {
                    hash: tx_hash,
                    index: *index as u32,
                };
                let sequence = if rbf { u32::max_value() - 2 } else { u32::max_value() };
                Ok(TransactionInput {
                    previous_output: outpoint,
                    script_sig: script_sig.to_bytes(),
                    sequence,
                    script_witness: vec![],
                })
            }).collect();
        let inputs = inputs?;
        let to_address = input_tx.to.clone().into_inner();
        let address_to: Address = to_address.parse().map_err(|e: BtcKeyError| {
            let e = format_err!("{}", e);
            ectx!(try err e, ErrorKind::MalformedAddress)
        })?;

        let address_to_hash = address_to.hash;

        let output_script = match address_to.kind {
            AddressType::P2PKH => ScriptBuilder::build_p2pkh(&address_to_hash),
            AddressType::P2SH => ScriptBuilder::build_p2sh(&address_to_hash),
        };

        let output = TransactionOutput {
            value: input_value,
            script_pubkey: output_script.to_bytes(),
        };
        let mut outputs = vec![output.clone()];
        let maybe_sum_inputs = utxos
            .iter()
            .fold(Some(Amount::new(0)), |acc, utxo| acc.and_then(|a| a.checked_add(utxo.value)));
        let sum_inputs = maybe_sum_inputs
            .and_then(|sum| sum.u64())
            .ok_or(ectx!(try err ErrorKind::Overflow, ErrorKind::Overflow))?;
        // Need to be strictly greater since we need to include fees as well
        if sum_inputs <= output.value {
            return Err(ectx!(err ErrorKind::NotEnoughUtxo, ErrorKind::NotEnoughUtxo => input_tx));
        };
        let rest = sum_inputs - output.value;
        let script = ScriptBuilder::build_p2pkh(&address_from_hash);
        let output = TransactionOutput {
            value: rest as u64,
            script_pubkey: script.to_bytes(),
        };
        outputs.push(output);
        let mut tx = Transaction {
            version: 1,
            inputs: inputs.clone(),
            outputs: outputs,
            lock_time: lock_time.unwrap_or(0),
        };
        // Estimating fees and deduct them from the last output (the one with address equal to input)
        let tx_raw = serialize(&tx).take();
        let fees = self
            .estimate_fees(input_tx.fee_price, inputs.len() as u64, tx_raw.len() as u64)
            .ok_or(ectx!(try err ErrorKind::Overflow, ErrorKind::Overflow))?;
        let outputs_len = tx.outputs.len();
        {
            let output_ref = tx
                .outputs
                .get_mut(outputs_len - 1)
                .expect("At least one output should always be in outputs");
            if fees >= output_ref.value {
                return Err(ectx!(err ErrorKind::NotEnoughUtxo, ErrorKind::NotEnoughUtxo => input_tx));
            }
            output_ref.value -= fees;
        };
        // Calculating signature to insert into inputs script
        let tx_raw = serialize(&tx).take();
        let mut tx_raw_with_sighash = tx_raw.clone();
        // SIGHASH_ALL
        tx_raw_with_sighash.extend([1, 0, 0, 0].iter());
        let tx_hash = sha256(&sha256(&tx_raw_with_sighash).take());
        let pk = hex_to_bytes(key.clone().into_inner())?;
        let pk = BtcPrivateKey::from_layout(&pk).map_err(|_| ectx!(try err ErrorContext::PrivateKeyConvert, ErrorKind::Internal => key))?;
        let keypair = KeyPair::from_private(pk).map_err(|_| ectx!(try err ErrorContext::PrivateKeyConvert, ErrorKind::Internal => key))?;
        let signature = keypair.private().sign(&tx_hash).map_err(|e| {
            let e = format_err!("{}", e);
            ectx!(try err e, ErrorContext::Signature, ErrorKind::Internal)
        })?;
        let mut signature_with_sighash = signature.to_vec();
        // SIGHASH_ALL
        signature_with_sighash.push(1);
        let public = keypair.public();
        let script = ScriptBuilder::default()
            .push_bytes(&signature_with_sighash)
            .push_bytes(&*public)
            .into_script();
        // Updating input_script to have signature
        for input_ref in tx.inputs.iter_mut() {
            input_ref.script_sig = script.to_bytes();
        }
        let tx_raw = serialize(&tx).take();
        let tx_raw_hex = bytes_to_hex(&tx_raw);
        Ok(RawTransaction::new(tx_raw_hex))
    }

    fn estimate_fees(&self, fee_price: Amount, inputs_count: u64, tx_size: u64) -> Option<u64> {
        let script_sig_size = 1 + 71 + 1 + 1 + 64;
        let script_pubkey_size = 3 + 20 + 2;
        let signature_bytes = (script_sig_size - script_pubkey_size) * inputs_count;
        let estimated_final_size = tx_size + signature_bytes;
        fee_price.u64().and_then(|fee| fee.checked_mul(estimated_final_size))
    }
}

impl BlockchainService for BitcoinService {
    // https://en.bitcoin.it/wiki/OP_CHECKSIG
    // https://bitcoin.stackexchange.com/questions/3374/how-to-redeem-a-basic-tx
    fn derive_address(&self, _currency: Currency, key: PrivateKey) -> Result<BlockchainAddress, Error> {
        let key_bytes = hex_to_bytes(key.into_inner()).map_err(|_| ectx!(try err ErrorContext::PrivateKeyConvert, ErrorKind::Internal))?;;
        let private: BtcPrivateKey =
            BtcPrivateKey::from_layout(&key_bytes).map_err(|_| ectx!(try err ErrorContext::PrivateKeyConvert, ErrorKind::Internal))?;
        let keypair = KeyPair::from_private(private).map_err(|_| ectx!(try err ErrorContext::PrivateKeyConvert, ErrorKind::Internal))?;
        Ok(BlockchainAddress::new(format!("{}", keypair.address())))
    }

    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error> {
        self.sign_with_options(key, tx, false, None)
    }

    fn approve(&self, _key: PrivateKey, _tx: ApproveInput) -> Result<RawTransaction, Error> {
        unimplemented!()
    }

    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error> {
        assert_eq!(currency, Currency::Btc, "unexpected currency: {:?}", currency);
        let network = match self.btc_network {
            BtcNetwork::Test => Network::Testnet,
            BtcNetwork::Main => Network::Mainnet,
        };
        let random = Random::new(network);
        let keypair = random.generate().map_err(|e| {
            let e = format_err!("{}", e);
            ectx!(try err e, ErrorSource::Random, ErrorKind::Internal)
        })?;
        let address = BlockchainAddress::new(format!("{}", keypair.address()));
        let pk_bytes = bytes_to_hex(&keypair.private().layout());
        let private_key = PrivateKey::new(pk_bytes);
        Ok((private_key, address))
    }
}

impl BitcoinService {
    pub fn new(btc_network: BtcNetwork) -> Self {
        BitcoinService { btc_network }
    }

    fn needed_utxos(&self, utxos: &[Utxo], value: Amount) -> Result<Vec<Utxo>, Error> {
        let mut utxos = utxos.to_vec();
        utxos.sort_by_key(|x| x.value);
        let mut res = Vec::new();
        let mut sum = Amount::new(0);
        for utxo in utxos.iter().rev() {
            res.push(utxo.clone());
            sum = sum
                .checked_add(utxo.value)
                .ok_or(ectx!(try err ErrorKind::Overflow, ErrorKind::Overflow => utxos, value))?;
            if sum >= value {
                return Ok(res);
            }
        }
        Err(ectx!(err ErrorKind::NotEnoughUtxo, ErrorKind::NotEnoughUtxo => utxos, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // https://testnet.blockchain.info/tx/5aed90d51d84d54d1093995f6d6a0e1e4503f40deefce942817bec6ad3cafe81?format=hex
    #[test]
    fn test_sign() {
        let bitcoin_service = BitcoinService::new(BtcNetwork::Test);
        let pk = PrivateKey::new("ef13c9b34216f7fbe84787ab9ff78f9fd516a1d72a78f071bfaaad97278fa86b5a9951c8c0".to_string());
        let tx = UnsignedTransaction {
            id: TransactionId::default(),
            from: BlockchainAddress::new("n4qX9Fh5wZopB1e2MGcpHUAy24NC7JxMwm".to_string()),
            to: BlockchainAddress::new("ms3iZko2BcbigHBufFUum2Avg9PfozmZY4".to_string()),
            currency: Currency::Btc,
            value: Amount::new(100000),
            fee_price: Amount::new(0),
            nonce: None,
            utxos: Some(vec![Utxo {
                tx_hash: "90e56bda920e72e9caae86302c284f18255a419927a0649fca839faeca1b8610".to_string(),
                value: Amount::new(8293863),
                index: 0,
            }]),
        };
        let raw_tx = bitcoin_service
            .sign_with_options(pk, tx, true, Some(1436452))
            .expect("Failed to sign");
        assert_eq!(raw_tx.into_inner(), "010000000110861bcaae9f83ca9f64a02799415a25184f282c3086aecae9720e92da6be590000000008a473044022065d8c5c83d1262e47447127aec29f78b80bce5cf8702f61679529019cc37bfa502204ca0377bd13ec7445b56e726c143f4da718e4424c2ec9acd68a58255f435992b0141049cd145484ef05dc259326651e942ecfa2c7f64bad3286e94e303eaf9b03edf0a844d63ad58c078e28a183438d0bccc75fd788522069ed79cee71736fade65124fdffffff02a0860100000000001976a9147e7ad15c2aa503c33520dee5bccd7d79ff2b44db88ac47077d00000000001976a914ffcdccfab05fa7df11e279da558d68f80daffc3788ac24eb1500".to_string());
    }
}
