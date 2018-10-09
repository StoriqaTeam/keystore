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

//! Façade crate for `patricia_trie` for Ethereum specific impls

pub extern crate patricia_trie as trie; // `pub` because we need to import this crate for the tests in `patricia_trie` and there were issues: https://gist.github.com/dvdplm/869251ee557a1b4bd53adc7c971979aa
extern crate elastic_array;
extern crate parity_bytes;
extern crate ethereum_types;
extern crate hashdb;
extern crate keccak_hasher;
extern crate rlp;

mod rlp_node_codec;

pub use rlp_node_codec::RlpNodeCodec;

use ethereum_types::H256;
use keccak_hasher::KeccakHasher;
use rlp::DecoderError;

/// Convenience type alias to instantiate a Keccak-flavoured `RlpNodeCodec`
pub type RlpCodec = RlpNodeCodec<KeccakHasher>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDB`
///
/// Use it as a `Trie` trait object. You can use `db()` to get the backing database object.
/// Use `get` and `contains` to query values associated with keys in the trie.
///
/// # Example
/// ```
/// extern crate patricia_trie as trie;
/// extern crate patricia_trie_ethereum as ethtrie;
/// extern crate hashdb;
/// extern crate keccak_hasher;
/// extern crate memorydb;
/// extern crate ethereum_types;
///
/// use trie::*;
/// use hashdb::*;
/// use keccak_hasher::KeccakHasher;
/// use memorydb::*;
/// use ethereum_types::H256;
/// use ethtrie::{TrieDB, TrieDBMut};
///
///
/// fn main() {
///   let mut memdb = MemoryDB::<KeccakHasher>::new();
///   let mut root = H256::new();
///   TrieDBMut::new(&mut memdb, &mut root).insert(b"foo", b"bar").unwrap();
///   let t = TrieDB::new(&memdb, &root).unwrap();
///   assert!(t.contains(b"foo").unwrap());
///   assert_eq!(t.get(b"foo").unwrap().unwrap(), DBValue::from_slice(b"bar"));
/// }
/// ```
pub type TrieDB<'db> = trie::TrieDB<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `SecTrieDB`
pub type SecTrieDB<'db> = trie::SecTrieDB<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `FatDB`
pub type FatDB<'db> = trie::FatDB<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBMut`
///
/// Use it as a `TrieMut` trait object. You can use `db()` to get the backing database object.
/// Note that changes are not committed to the database until `commit` is called.
/// Querying the root or dropping the trie will commit automatically.

/// # Example
/// ```
/// extern crate patricia_trie as trie;
/// extern crate patricia_trie_ethereum as ethtrie;
/// extern crate hashdb;
/// extern crate keccak_hash;
/// extern crate keccak_hasher;
/// extern crate memorydb;
/// extern crate ethereum_types;
///
/// use keccak_hash::KECCAK_NULL_RLP;
/// use ethtrie::{TrieDBMut, trie::TrieMut};
/// use hashdb::DBValue;
/// use keccak_hasher::KeccakHasher;
/// use memorydb::*;
/// use ethereum_types::H256;
///
/// fn main() {
///   let mut memdb = MemoryDB::<KeccakHasher>::new();
///   let mut root = H256::new();
///   let mut t = TrieDBMut::new(&mut memdb, &mut root);
///   assert!(t.is_empty());
///   assert_eq!(*t.root(), KECCAK_NULL_RLP);
///   t.insert(b"foo", b"bar").unwrap();
///   assert!(t.contains(b"foo").unwrap());
///   assert_eq!(t.get(b"foo").unwrap().unwrap(), DBValue::from_slice(b"bar"));
///   t.remove(b"foo").unwrap();
///   assert!(!t.contains(b"foo").unwrap());
/// }
/// ```
pub type TrieDBMut<'db> = trie::TrieDBMut<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `SecTrieDBMut`
pub type SecTrieDBMut<'db> = trie::SecTrieDBMut<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `FatDBMut`
pub type FatDBMut<'db> = trie::FatDBMut<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieFactory`
pub type TrieFactory = trie::TrieFactory<KeccakHasher, RlpCodec>;

/// Convenience type alias for Keccak/Rlp flavoured trie errors
pub type TrieError = trie::TrieError<H256, DecoderError>;
/// Convenience type alias for Keccak/Rlp flavoured trie results
pub type Result<T> = trie::Result<T, H256, DecoderError>;
