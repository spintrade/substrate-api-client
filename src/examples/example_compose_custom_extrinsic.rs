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

#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate substrate_api_client;

use clap::App;
use indices::address::Address;
use keyring::AccountKeyring;
use node_primitives::AccountId;
use parity_codec::Compact;
use parity_codec::Encode;
use primitive_types::U256;
use primitives::offchain::CryptoKind;
use node_primitives::Balance;


use substrate_api_client::{Api, extrinsic};
use substrate_api_client::compose;
use substrate_api_client::utils::hexstr_to_u256;

fn main() {
    env_logger::init();

    let yml = load_yaml!("../../src/examples/cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}", url);

    let mut api = Api::new(format!("ws://{}", url));
    api.init();

    // get Alice's AccountNonce
    let accountid = AccountId::from(AccountKeyring::Alice);
    let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
    let nonce = hexstr_to_u256(result_str);
    println!("[+] Alice's Account Nonce is {}", nonce);

    // generate extrinsic
//    let xt: Vec<u8> = compose!(api.metadata.clone(),
//                                 api.genesis_hash.unwrap(),
//                                CryptoKind::Sr25519,
//                                "substratee_registry",
//                                "confirm_call",
//                                nonce,
//                                "//Alice",
//                               vec![9u8; 2],
//                               vec![2u8; 2]);


    let to = extrinsic::crypto::AccountKey::public_from_suri("//Bob", Some(""), CryptoKind::Sr25519);
    let to = Address::<[u8; 32], u32>::from(to);
    let xt = compose!(api.metadata.clone(),
                                 api.genesis_hash.unwrap(),
                                CryptoKind::Sr25519,
                                "balances",
                                "transfer",
                                nonce,
                                "//Alice",
                                to,
                                Compact(Balance::from(42 as u128)));

    let mut _xthex = hex::encode(xt.encode());
    _xthex.insert_str(0, "0x");
    //send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(_xthex).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
}