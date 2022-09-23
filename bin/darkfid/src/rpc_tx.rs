use std::str::FromStr;

use log::{error, warn};
use serde_json::{json, Value};

use darkfi::{
    consensus::ValidatorState,
    crypto::{address::Address, keypair::PublicKey, token_id},
    node::MemoryState,
    rpc::jsonrpc::{ErrorCode::InvalidParams, JsonError, JsonResponse, JsonResult},
    tx::Transaction,
    util::serial::{deserialize, serialize},
};

use super::Darkfid;
use crate::{server_error, RpcError};

impl Darkfid {
    // RPCAPI:
    // Transfer a given amount of some token to the given address.
    // Returns a transaction ID upon success.
    //
    // * `dest_addr` -> Recipient's DarkFi address
    // * `token_id` -> ID of the token to send
    // * `12345` -> Amount in `u64` of the funds to send
    //
    // --> {"jsonrpc": "2.0", "method": "tx.transfer", "params": ["dest_addr", "token_id", 12345], "id": 1}
    // <-- {"jsonrpc": "2.0", "result": "txID...", "id": 1}
    pub async fn transfer(&self, id: Value, params: &[Value]) -> JsonResult {
        if params.len() != 3 ||
            !params[0].is_string() ||
            !params[1].is_string() ||
            !params[2].is_u64()
        {
            return JsonError::new(InvalidParams, None, id).into()
        }

        if !(*self.synced.lock().await) {
            error!("tx.transfer: Blockchain is not yet synced");
            return server_error(RpcError::NotYetSynced, id)
        }

        let address = params[0].as_str().unwrap();
        let token = params[1].as_str().unwrap();
        let amount = params[2].as_u64().unwrap();

        let address = match Address::from_str(address) {
            Ok(v) => v,
            Err(e) => {
                error!("tx.transfer: Failed parsing address from string: {}", e);
                return server_error(RpcError::InvalidAddressParam, id)
            }
        };

        let pubkey = match PublicKey::try_from(address) {
            Ok(v) => v,
            Err(e) => {
                error!("tx.transfer: Failed parsing PublicKey from Address: {}", e);
                return server_error(RpcError::ParseError, id)
            }
        };

        let token_id = match token_id::parse_b58(token) {
            Ok(v) => v,
            Err(e) => {
                error!("tx.transfer: Failed parsing Token ID from string: {}", e);
                return server_error(RpcError::ParseError, id)
            }
        };

        let tx = match self
            .client
            .build_transaction(
                pubkey,
                amount,
                token_id,
                false,
                self.validator_state.read().await.state_machine.clone(),
            )
            .await
        {
            Ok(v) => v,
            Err(e) => {
                error!("tx.transfer: Failed building transaction: {}", e);
                return server_error(RpcError::TxBuildFail, id)
            }
        };

        if let Some(sync_p2p) = &self.sync_p2p {
            if let Err(e) = sync_p2p.broadcast(tx.clone()).await {
                error!("tx.transfer: Failed broadcasting transaction: {}", e);
                return server_error(RpcError::TxBroadcastFail, id)
            }
        } else {
            warn!("tx.transfer: No sync P2P network, not broadcasting transaction.");
            return server_error(RpcError::TxBroadcastFail, id)
        }

        let tx_hash = blake3::hash(&serialize(&tx)).to_hex().as_str().to_string();
        JsonResponse::new(json!(tx_hash), id).into()
    }

    // RPCAPI:
    // Broadcast a given transaction to the P2P network.
    // The function will first simulate the state transition in order to see
    // if the transaction is actually valid, and in turn it will return an
    // error if this is the case. Otherwise, a transaction ID will be returned.
    //
    // --> {"jsonrpc": "2.0", "method": "tx.broadcast", "params": ["base58encodedTX"], "id": 1}
    // <-- {"jsonrpc": "2.0", "result": "txID...", "id": 1}
    pub async fn broadcast(&self, id: Value, params: &[Value]) -> JsonResult {
        if params.len() != 1 || params[0].is_string() {
            return JsonError::new(InvalidParams, None, id).into()
        }

        if !(*self.synced.lock().await) {
            error!("tx.transfer: Blockchain is not yet synced");
            return server_error(RpcError::NotYetSynced, id)
        }

        // Try to deserialize the transaction
        let tx_bytes = match bs58::decode(params[0].as_str().unwrap()).into_vec() {
            Ok(v) => v,
            Err(e) => {
                error!("tx.broadcast: Failed decoding base58 transaction: {}", e);
                return server_error(RpcError::ParseError, id)
            }
        };

        let tx: Transaction = match deserialize(&tx_bytes) {
            Ok(v) => v,
            Err(e) => {
                error!("tx.broadcast: Failed deserializing bytes into Transaction: {}", e);
                return server_error(RpcError::ParseError, id)
            }
        };

        // Grab the current state and apply a new MemoryState
        let state_machine = self.validator_state.read().await.state_machine.lock().await.clone();
        let mem_state = MemoryState::new(state_machine.clone());
        drop(state_machine);

        // Simulate state transition
        if let Err(e) = ValidatorState::validate_state_transitions(mem_state, &[tx.clone()]) {
            error!("tx.broadcast: Failed to validate state transition: {}", e);
            return server_error(RpcError::TxSimulationFail, id)
        }

        if let Some(sync_p2p) = &self.sync_p2p {
            if let Err(e) = sync_p2p.broadcast(tx.clone()).await {
                error!("tx.broadcast: Failed broadcasting transaction: {}", e);
                return server_error(RpcError::TxBroadcastFail, id)
            }
        } else {
            warn!("tx.broadcast: No sync P2P network, not broadcasting transaction.");
            return server_error(RpcError::TxBroadcastFail, id)
        }

        let tx_hash = blake3::hash(&serialize(&tx)).to_hex().as_str().to_string();
        JsonResponse::new(json!(tx_hash), id).into()
    }
}