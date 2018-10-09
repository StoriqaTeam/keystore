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

//! Tendermint message handling.

use bytes::Bytes;
use error::Error;
use ethereum_types::{H256, H520, Address};
use ethkey::{recover, public_to_address};
use hash::keccak;
use header::Header;
use rlp::{Rlp, RlpStream, Encodable, Decodable, DecoderError};
use std::cmp;
use super::{Height, View, BlockHash, Step};
use super::super::vote_collector::Message;

/// Message transmitted between consensus participants.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub struct ConsensusMessage {
	pub vote_step: VoteStep,
	pub block_hash: Option<BlockHash>,
	pub signature: H520,
}

/// Complete step of the consensus process.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct VoteStep {
	pub height: Height,
	pub view: View,
	pub step: Step,
}

impl VoteStep {
	pub fn new(height: Height, view: View, step: Step) -> Self {
		VoteStep { height: height, view: view, step: step }
	}

	pub fn is_height(&self, height: Height) -> bool {
		self.height == height
	}

	pub fn is_view(&self, height: Height, view: View) -> bool {
		self.height == height && self.view == view
	}
}

/// Header consensus view.
pub fn consensus_view(header: &Header) -> Result<View, ::rlp::DecoderError> {
	let view_rlp = header.seal().get(0).expect("seal passed basic verification; seal has 3 fields; qed");
	Rlp::new(view_rlp.as_slice()).as_val()
}

/// Proposal signature.
pub fn proposal_signature(header: &Header) -> Result<H520, ::rlp::DecoderError> {
	Rlp::new(header.seal().get(1).expect("seal passed basic verification; seal has 3 fields; qed").as_slice()).as_val()
}

impl Message for ConsensusMessage {
	type Round = VoteStep;

	fn signature(&self) -> H520 { self.signature }

	fn block_hash(&self) -> Option<H256> { self.block_hash }

	fn round(&self) -> &VoteStep { &self.vote_step }

	fn is_broadcastable(&self) -> bool { self.vote_step.step.is_pre() }
}

impl ConsensusMessage {
	pub fn new(signature: H520, height: Height, view: View, step: Step, block_hash: Option<BlockHash>) -> Self {
		ConsensusMessage {
			signature: signature,
			block_hash: block_hash,
			vote_step: VoteStep::new(height, view, step),
		}
	}

	pub fn new_proposal(header: &Header) -> Result<Self, ::rlp::DecoderError> {
		Ok(ConsensusMessage {
			signature: proposal_signature(header)?,
			vote_step: VoteStep::new(header.number() as Height, consensus_view(header)?, Step::Propose),
			block_hash: Some(header.bare_hash()),
		})
	}

	pub fn verify(&self) -> Result<Address, Error> {
		let full_rlp = ::rlp::encode(self);
		let block_info = Rlp::new(&full_rlp).at(1)?;
		let public_key = recover(&self.signature.into(), &keccak(block_info.as_raw()))?;
		Ok(public_to_address(&public_key))
	}
}

impl Default for VoteStep {
	fn default() -> Self {
		VoteStep::new(0, 0, Step::Propose)
	}
}

impl PartialOrd for VoteStep {
	fn partial_cmp(&self, m: &VoteStep) -> Option<cmp::Ordering> {
		Some(self.cmp(m))
	}
}

impl Ord for VoteStep {
	fn cmp(&self, m: &VoteStep) -> cmp::Ordering {
		if self.height != m.height {
			self.height.cmp(&m.height)
		} else if self.view != m.view {
			self.view.cmp(&m.view)
		} else {
			self.step.number().cmp(&m.step.number())
		}
	}
}

impl Step {
	fn number(&self) -> u8 {
		match *self {
			Step::Propose => 0,
			Step::Prevote => 1,
			Step::Precommit => 2,
			Step::Commit => 3,
		}
	}
}

impl Decodable for Step {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		match rlp.as_val()? {
			0u8 => Ok(Step::Propose),
			1 => Ok(Step::Prevote),
			2 => Ok(Step::Precommit),
			_ => Err(DecoderError::Custom("Invalid step.")),
		}
	}
}

impl Encodable for Step {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append_internal(&self.number());
	}
}

/// (signature, (height, view, step, block_hash))
impl Decodable for ConsensusMessage {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		let m = rlp.at(1)?;
		let block_message: H256 = m.val_at(3)?;
		Ok(ConsensusMessage {
			vote_step: VoteStep::new(m.val_at(0)?, m.val_at(1)?, m.val_at(2)?),
			block_hash: match block_message.is_zero() {
				true => None,
				false => Some(block_message),
			},
			signature: rlp.val_at(0)?,
		})
  }
}

impl Encodable for ConsensusMessage {
	fn rlp_append(&self, s: &mut RlpStream) {
		let info = message_info_rlp(&self.vote_step, self.block_hash);
		s.begin_list(2)
			.append(&self.signature)
			.append_raw(&info, 1);
	}
}

pub fn message_info_rlp(vote_step: &VoteStep, block_hash: Option<BlockHash>) -> Bytes {
	let mut s = RlpStream::new_list(4);
	s.append(&vote_step.height).append(&vote_step.view).append(&vote_step.step).append(&block_hash.unwrap_or_else(H256::zero));
	s.out()
}

pub fn message_full_rlp(signature: &H520, vote_info: &Bytes) -> Bytes {
	let mut s = RlpStream::new_list(2);
	s.append(signature).append_raw(vote_info, 1);
	s.out()
}

pub fn message_hash(vote_step: VoteStep, block_hash: H256) -> H256 {
	keccak(message_info_rlp(&vote_step, Some(block_hash)))
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use hash::keccak;
	use rlp::*;
	use account_provider::AccountProvider;
	use header::Header;
	use super::super::Step;
	use super::*;

	#[test]
	fn encode_step() {
		let step = Step::Precommit;

		let mut s = RlpStream::new_list(2);
		s.append(&step);
		assert!(!s.is_finished(), "List shouldn't finished yet");
		s.append(&step);
		assert!(s.is_finished(), "List should be finished now");
		s.out();
	}

	#[test]
	fn encode_decode() {
		let message = ConsensusMessage {
			signature: H520::default(),
			vote_step: VoteStep {
				height: 10,
				view: 123,
				step: Step::Precommit,
			},
			block_hash: Some(keccak("1")),
		};
		let raw_rlp = ::rlp::encode(&message).into_vec();
		let rlp = Rlp::new(&raw_rlp);
		assert_eq!(Ok(message), rlp.as_val());

		let message = ConsensusMessage {
			signature: H520::default(),
			vote_step: VoteStep {
				height: 1314,
				view: 0,
				step: Step::Prevote,
			},
			block_hash: None
		};
		let raw_rlp = ::rlp::encode(&message);
		let rlp = Rlp::new(&raw_rlp);
		assert_eq!(Ok(message), rlp.as_val());
	}

	#[test]
	fn generate_and_verify() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let addr = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();
		tap.unlock_account_permanently(addr, "0".into()).unwrap();

		let mi = message_info_rlp(&VoteStep::new(123, 2, Step::Precommit), Some(H256::default()));

		let raw_rlp = message_full_rlp(&tap.sign(addr, None, keccak(&mi)).unwrap().into(), &mi);

		let rlp = Rlp::new(&raw_rlp);
		let message: ConsensusMessage = rlp.as_val().unwrap();
		match message.verify() { Ok(a) if a == addr => {}, _ => panic!(), };
	}

	#[test]
	fn proposal_message() {
		let mut header = Header::default();
		let seal = vec![
			::rlp::encode(&0u8).into_vec(),
			::rlp::encode(&H520::default()).into_vec(),
			Vec::new()
		];

		header.set_seal(seal);
		let message = ConsensusMessage::new_proposal(&header).unwrap();
		assert_eq!(
			message,
			ConsensusMessage {
				signature: Default::default(),
				vote_step: VoteStep {
					height: 0,
					view: 0,
					step: Step::Propose,
				},
				block_hash: Some(header.bare_hash())
			}
		);
	}

	#[test]
	fn step_ordering() {
			assert!(VoteStep::new(10, 123, Step::Precommit) < VoteStep::new(11, 123, Step::Precommit));
			assert!(VoteStep::new(10, 123, Step::Propose) < VoteStep::new(11, 123, Step::Precommit));
			assert!(VoteStep::new(10, 122, Step::Propose) < VoteStep::new(11, 123, Step::Propose));
	}
}
