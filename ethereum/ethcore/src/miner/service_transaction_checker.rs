// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! A service transactions contract checker.

use client::{RegistryInfo, CallContract, BlockId};
use transaction::SignedTransaction;

use_contract!(service_transaction, "ServiceTransaction", "res/contracts/service_transaction.json");

const SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME: &'static str = "service_transaction_checker";

/// Service transactions checker.
#[derive(Default, Clone)]
pub struct ServiceTransactionChecker {
	contract: service_transaction::ServiceTransaction,
}

impl ServiceTransactionChecker {
	/// Checks if given address is whitelisted to send service transactions.
	pub fn check<C: CallContract + RegistryInfo>(&self, client: &C, tx: &SignedTransaction) -> Result<bool, String> {
		let sender = tx.sender();
		let hash = tx.hash();

		// Skip checking the contract if the transaction does not have zero gas price
		if !tx.gas_price.is_zero() {
			return Ok(false)
		}

		let address = client.registry_address(SERVICE_TRANSACTION_CONTRACT_REGISTRY_NAME.to_owned(), BlockId::Latest)
			.ok_or_else(|| "contract is not configured")?;

		trace!(target: "txqueue", "[{:?}] Checking service transaction checker contract from {}", hash, sender);

		self.contract.functions()
			.certified()
			.call(sender, &|data| client.call_contract(BlockId::Latest, address, data))
			.map_err(|e| e.to_string())
	}
}
