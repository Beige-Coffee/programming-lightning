#![allow(dead_code, unused_imports, unused_variables, unused_must_use)]
use crate::internal;
use internal::convert::BlockchainInfo;
use crate::ch2_setup::helpers::{get_http_endpoint, format_rpc_credentials, 
                                new_rpc_client, test_rpc_call, get_best_block,
                                get_chain_poller, get_new_cache, get_spv_client, ToHex
};
use base64;
use bitcoin::hash_types::{BlockHash};
use bitcoin::{Network};
use lightning_block_sync::http::HttpEndpoint;
use lightning_block_sync::rpc::RpcClient;
use lightning_block_sync::{AsyncBlockSourceResult, BlockData, BlockHeaderData, BlockSource};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use lightning::chain::Listen;
use lightning_block_sync::init::validate_best_block_header;
use lightning_block_sync::poll::ChainPoller;
use lightning_block_sync::SpvClient;
use lightning::chain::chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator};
use bitcoin::blockdata::transaction::Transaction;

#[derive(Clone)]
pub struct BitcoindClientExercise {
    pub(crate) bitcoind_rpc_client: Arc<RpcClient>,
    network: Network,
    host: String,
    port: u16,
    rpc_user: String,
    rpc_password: String
}

impl BitcoindClientExercise {
    async fn send_to_network(&self, tx_hex: String) -> bool {
        self.bitcoind_rpc_client
            .call_method::<serde_json::Value>("sendrawtransaction", &[tx_hex.into()])
            .await
            .is_ok()
    }
}

impl BlockSource for BitcoindClientExercise {
    fn get_header<'a>(
        &'a self, header_hash: &'a BlockHash, height_hint: Option<u32>,
    ) -> AsyncBlockSourceResult<'a, BlockHeaderData> {
        Box::pin(async move { 
            
            self.bitcoind_rpc_client.get_header(header_hash, height_hint).await
        
        })
    }

    fn get_block<'a>(
        &'a self, header_hash: &'a BlockHash,
    ) -> AsyncBlockSourceResult<'a, BlockData> {
        Box::pin(async move {
            
            self.bitcoind_rpc_client.get_block(header_hash).await
        
        })
    }

    fn get_best_block(&self) -> AsyncBlockSourceResult<(BlockHash, Option<u32>)> {
        Box::pin(async move { 
            
            self.bitcoind_rpc_client.get_best_block().await
        
        })
    }
}

impl BitcoindClientExercise {
    pub(crate) async fn new(
        host: String, port: u16, rpc_user: String, rpc_password: String, network: Network,
    ) -> std::io::Result<Self> {
        let http_endpoint = get_http_endpoint(&host, port);
        let rpc_credentials = format_rpc_credentials(&rpc_user, &rpc_password);
        let bitcoind_rpc_client = new_rpc_client(&rpc_credentials, http_endpoint);
        test_rpc_call(&bitcoind_rpc_client);

        let client = Self {
            bitcoind_rpc_client: bitcoind_rpc_client,
            host,
            port,
            rpc_user,
            rpc_password,
            network
        };

        Ok(client)
    }
}

pub async fn poll_for_blocks<L: Listen>(bitcoind: BitcoindClientExercise, network: Network,
                   listener: L) {

    let best_block_header = validate_best_block_header(&bitcoind).await.unwrap();

    let poller = ChainPoller::new(&bitcoind, network);

    let mut cache = HashMap::new();

    let mut spv_client = SpvClient::new(best_block_header, poller, & mut cache, &listener);

    loop {
        let best_block = spv_client.poll_best_tip().await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub async fn poll_for_blocks2<L: Listen>(bitcoind: BitcoindClientExercise, network: Network,
                   listener: L) {

    let best_block_header = get_best_block(bitcoind.clone());

    let poller = get_chain_poller(bitcoind, network);

    let mut cache = get_new_cache();

    let mut spv_client = get_spv_client(best_block_header.await, poller, &mut cache, &listener);

    loop {
        let best_block = spv_client.poll_best_tip().await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

impl BroadcasterInterface for BitcoindClientExercise {
    fn broadcast_transactions(&self, txs: &[&Transaction]) {
        for tx in txs {
            let hex_string = tx.to_hex();
            self.send_to_network(hex_string);
        }
    }
}