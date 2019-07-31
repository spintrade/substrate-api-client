/*
   Copyright 2019 Supercomputing Systems AG

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

*/

use indices::address::Address;
use node_primitives::{Balance, Hash, Index, Signature};
use parity_codec::{Compact, Encode};
use primitive_types::U256;
use primitives::{/*ed25519, */blake2_256, hexdisplay::HexDisplay, Pair};
use runtime_primitives::generic::{Era, UncheckedMortalCompactExtrinsic};

use crypto::{Crypto, Sr25519};

type UncheckedExtrinsic = UncheckedMortalCompactExtrinsic<Address<[u8; 32], u32>, Index, MyCall, Signature>;

mod crypto;

#[derive(Debug, Encode, PartialEq)]
pub enum MyCall {
	_Test(i16),
	_Test2(u16),
	_Test3(u32),
	// In our current setup, the Balances module is the fourth  listed, which does expose calls.
	// Hence it needs to be listed as fourth variant in an enum to be encoded correctly.
	Balances(Balances),
}

#[derive(Debug, Encode, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Balances {
	transfer(Address<[u8; 32], u32>, Compact<u128>),
}


// see https://wiki.parity.io/Extrinsic
pub fn transfer(from: &str, to: &str, amount: U256, index: U256, genesis_hash: Hash) -> UncheckedExtrinsic {
	let to = Sr25519::public_from_suri(to, Some(""));

	let amount = Balance::from(amount.low_u128());
	let function = MyCall::Balances(Balances::transfer(Address::from(to.0), Compact(amount)));
	compose_extrinsic(from, function, index, genesis_hash)
}

pub fn compose_extrinsic(from: &str, function: MyCall, index: U256, genesis_hash: Hash) -> UncheckedExtrinsic {
	debug!("using genesis hash: {:?}", genesis_hash);

	let signer = Sr25519::pair_from_suri(from, Some(""));
	let era = Era::immortal();

	let index = Index::from(index.low_u64());

	let raw_payload = (Compact(index), function, era, genesis_hash);
	let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
		signer.sign(&blake2_256(payload)[..])
	} else {
		debug!("signing {}", HexDisplay::from(&payload));
		signer.sign(payload)
	});

	UncheckedExtrinsic {
		signature: Some((Address::from(signer.public().0), signature.into(), index.into(), era)),
		function: raw_payload.1,
	}
}

// pub fn sign(xt: CheckedExtrinsic, key: &sr25519::Pair, genesis_hash: Hash) -> UncheckedExtrinsic {
// 	match xt.signed {
// 		Some((signed, index)) => {
// 			let era = Era::immortal();
// 			let payload = (index.into(), xt.function, era, genesis_hash);
// 			assert_eq!(key.public(), signed);
// 			let signature = payload.using_encoded(|b| {
// 				if b.len() > 256 {
// 					key.sign(&blake2_256(b))
// 				} else {
// 					key.sign(b)
// 				}
// 			}).into();
// 			UncheckedExtrinsic {
// 				signature: Some((signed.into(), signature, payload.0, era)),
// 				function: payload.1,
// 			}
// 		}
// 		None => UncheckedExtrinsic {
// 			signature: None,
// 			function: xt.function,
// 		},
// 	}
// }

#[cfg(test)]
mod tests {
	use node_runtime::{BalancesCall, Call};
	use primitive_types::U128;

	use super::*;

	#[test]
	fn custom_call_encoded_equals_imported_call() {
		let amount = Balance::from(42 as u128);

		let to = sr25519::Pair::from_string("//Alice", Some("")).ok().map(|p| p.public())
			.expect("Invalid URI; expecting either a secret URI or a public URI.");


		let my_call = MyCall::Balances(balances::transfer(to.clone().into(), Compact(amount))).encode();
		let balances_call = Call::Balances(BalancesCall::transfer(to.clone().into(), amount)).encode();
		assert_eq!(my_call, balances_call);
	}
}