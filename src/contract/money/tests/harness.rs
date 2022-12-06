/* This file is part of DarkFi (https://dark.fi)
 *
 * Copyright (C) 2020-2022 Dyne.org foundation
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::collections::HashMap;

use darkfi::{
    consensus::{
        constants::{TESTNET_GENESIS_HASH_BYTES, TESTNET_GENESIS_TIMESTAMP},
        ValidatorState, ValidatorStatePtr,
    },
    wallet::WalletDb,
    zk::{proof::ProvingKey, vm::ZkCircuit, vm_stack::empty_witnesses},
    zkas::ZkBinary,
    Result,
};
use darkfi_sdk::{
    crypto::{constants::MERKLE_DEPTH, ContractId, Keypair, MerkleNode, PublicKey},
    db::ZKAS_DB_NAME,
    incrementalmerkletree::bridgetree::BridgeTree,
    pasta::{group::ff::PrimeField, pallas},
};
use darkfi_serial::serialize;
use log::info;
use rand::rngs::OsRng;

use darkfi_money_contract::{ZKAS_BURN_NS, ZKAS_MINT_NS};

pub struct MoneyTestHarness {
    pub faucet_kp: Keypair,
    pub alice_kp: Keypair,
    pub bob_kp: Keypair,
    pub faucet_pubkeys: Vec<PublicKey>,
    pub faucet_state: ValidatorStatePtr,
    pub alice_state: ValidatorStatePtr,
    pub bob_state: ValidatorStatePtr,
    pub money_contract_id: ContractId,
    pub proving_keys: HashMap<[u8; 32], Vec<(&'static str, ProvingKey)>>,
    pub mint_zkbin: ZkBinary,
    pub burn_zkbin: ZkBinary,
    pub mint_pk: ProvingKey,
    pub burn_pk: ProvingKey,
    pub faucet_merkle_tree: BridgeTree<MerkleNode, MERKLE_DEPTH>,
    pub alice_merkle_tree: BridgeTree<MerkleNode, MERKLE_DEPTH>,
    pub bob_merkle_tree: BridgeTree<MerkleNode, MERKLE_DEPTH>,
}

impl MoneyTestHarness {
    pub async fn new() -> Result<Self> {
        let faucet_kp = Keypair::random(&mut OsRng);
        let alice_kp = Keypair::random(&mut OsRng);
        let bob_kp = Keypair::random(&mut OsRng);
        let faucet_pubkeys = vec![faucet_kp.public];

        let faucet_wallet = WalletDb::new("sqlite::memory:", "foo").await?;
        let alice_wallet = WalletDb::new("sqlite::memory:", "foo").await?;
        let bob_wallet = WalletDb::new("sqlite::memory:", "foo").await?;

        let faucet_sled_db = sled::Config::new().temporary(true).open()?;
        let alice_sled_db = sled::Config::new().temporary(true).open()?;
        let bob_sled_db = sled::Config::new().temporary(true).open()?;

        let faucet_state = ValidatorState::new(
            &faucet_sled_db,
            *TESTNET_GENESIS_TIMESTAMP,
            *TESTNET_GENESIS_HASH_BYTES,
            faucet_wallet,
            faucet_pubkeys.clone(),
            false,
        )
        .await?;

        let alice_state = ValidatorState::new(
            &alice_sled_db,
            *TESTNET_GENESIS_TIMESTAMP,
            *TESTNET_GENESIS_HASH_BYTES,
            alice_wallet,
            faucet_pubkeys.clone(),
            false,
        )
        .await?;

        let bob_state = ValidatorState::new(
            &bob_sled_db,
            *TESTNET_GENESIS_TIMESTAMP,
            *TESTNET_GENESIS_HASH_BYTES,
            bob_wallet,
            faucet_pubkeys.clone(),
            false,
        )
        .await?;

        let money_contract_id = ContractId::from(pallas::Base::from(u64::MAX - 420));

        let alice_sled = alice_state.read().await.blockchain.sled_db.clone();
        let db_handle = alice_state.read().await.blockchain.contracts.lookup(
            &alice_sled,
            &money_contract_id,
            ZKAS_DB_NAME,
        )?;

        let mint_zkbin = db_handle.get(&serialize(&ZKAS_MINT_NS))?.unwrap();
        let burn_zkbin = db_handle.get(&serialize(&ZKAS_BURN_NS))?.unwrap();
        info!("Decoding bincode");
        let mint_zkbin = ZkBinary::decode(&mint_zkbin.clone())?;
        let burn_zkbin = ZkBinary::decode(&burn_zkbin.clone())?;
        let mint_witnesses = empty_witnesses(&mint_zkbin);
        let burn_witnesses = empty_witnesses(&burn_zkbin);
        let mint_circuit = ZkCircuit::new(mint_witnesses, mint_zkbin.clone());
        let burn_circuit = ZkCircuit::new(burn_witnesses, burn_zkbin.clone());

        info!("Creating zk proving keys");
        let k = 13;
        let mut proving_keys = HashMap::<[u8; 32], Vec<(&str, ProvingKey)>>::new();
        let mint_pk = ProvingKey::build(k, &mint_circuit);
        let burn_pk = ProvingKey::build(k, &burn_circuit);
        let pks = vec![(ZKAS_MINT_NS, mint_pk.clone()), (ZKAS_BURN_NS, burn_pk.clone())];
        proving_keys.insert(money_contract_id.inner().to_repr(), pks);

        let faucet_merkle_tree = BridgeTree::<MerkleNode, MERKLE_DEPTH>::new(100);
        let alice_merkle_tree = BridgeTree::<MerkleNode, MERKLE_DEPTH>::new(100);
        let bob_merkle_tree = BridgeTree::<MerkleNode, MERKLE_DEPTH>::new(100);

        Ok(Self {
            faucet_kp,
            alice_kp,
            bob_kp,
            faucet_pubkeys,
            faucet_state,
            alice_state,
            bob_state,
            money_contract_id,
            proving_keys,
            mint_pk: mint_pk.clone(),
            burn_pk: burn_pk.clone(),
            mint_zkbin,
            burn_zkbin,
            faucet_merkle_tree,
            alice_merkle_tree,
            bob_merkle_tree,
        })
    }
}