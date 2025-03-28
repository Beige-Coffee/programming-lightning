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
use tokio::time::{sleep, Duration};
use lightning::chain::Listen;
use lightning_block_sync::init::validate_best_block_header;
use lightning_block_sync::poll::ChainPoller;
use lightning_block_sync::SpvClient;
use lightning::chain::chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator};
use lightning::chain::chaininterface::ConfirmationTarget::{
    MaximumFeeEstimate,
    UrgentOnChainSweep,
    MinAllowedAnchorChannelRemoteFee,
    MinAllowedNonAnchorChannelRemoteFee,
    AnchorChannelFee,
    NonAnchorChannelFee,
    ChannelCloseMinimum,
    OutputSpendingFee,
};
use bitcoin::consensus::{encode, Decodable, Encodable};
use bitcoin::blockdata::transaction::Transaction;
use tokio::runtime::Handle;
use crate::internal::convert::ListUnspentResponse;

//
// Exercise 1 (START)
//

#[derive(Clone)]
pub struct BitcoinClient {
    pub(crate) bitcoind_rpc_client: Arc<RpcClient>,
    network: Network,
    host: String,
    port: u16,
    rpc_user: String,
    rpc_password: String,
    pub handle: tokio::runtime::Handle,
}

impl BitcoinClient {
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
                network,
                handle: tokio::runtime::Handle::current()
            };

            Ok(client)
        }
}

//
// Exercise 1 (END)
//

//----------------------------------------------------------------------

//
// Exercise 2A: BlockSource (START)
//

impl BlockSource for BitcoinClient {
    fn get_header<'a>(
        &'a self, header_hash: &'a BlockHash, height_hint: Option<u32>,
    ) -> AsyncBlockSourceResult<'a, BlockHeaderData> {
        Box::pin(async move { self.bitcoind_rpc_client.get_header(header_hash, height_hint).await })
    }

    fn get_block<'a>(
        &'a self, header_hash: &'a BlockHash,
    ) -> AsyncBlockSourceResult<'a, BlockData> {
        Box::pin(async move { self.bitcoind_rpc_client.get_block(header_hash).await })
    }

    fn get_best_block(&self) -> AsyncBlockSourceResult<(BlockHash, Option<u32>)> {
        Box::pin(async move { self.bitcoind_rpc_client.get_best_block().await })
    }
}

//
// Exercise 2A: BlockSource (END)
//

//----------------------------------------------------------------------

//
// Exercise 2B: List Unspent (START)
//

impl BitcoinClient {
    pub async fn get_blockchain_info(&self) -> BlockchainInfo {
        self.bitcoind_rpc_client
            .call_method::<BlockchainInfo>("getblockchaininfo", &vec![])
            .await
            .unwrap()
    }

    pub async fn list_unspent(&self) -> ListUnspentResponse {
        self.bitcoind_rpc_client
            .call_method::<ListUnspentResponse>("listunspent", &vec![])
            .await
            .unwrap()
    }
}

//
// Exercise 2B: List Unspent (END)
//

//----------------------------------------------------------------------

//
// Exercise 3: BroadcasterInterface (START)
//

impl BitcoinClient {
    fn sendrawtransaction(&self, tx_hex: String) {
        let bitcoind_rpc_client = self.bitcoind_rpc_client.clone();
        self.handle.spawn(async move {
            let tx_json = serde_json::json!(tx_hex);

            if let Err(e) = bitcoind_rpc_client
                .call_method::<serde_json::Value>("sendrawtransaction", &[tx_json])
                .await
            {
                eprintln!("Failed to broadcast transaction: {}", e);
            } else {
                println!("Successfully broadcasted transaction: {}", tx_hex);
            }
        });
    }
}

impl BroadcasterInterface for BitcoinClient {
    fn broadcast_transactions(&self, txs: &[&Transaction]) {
        for tx in txs {
            let tx_hex = encode::serialize_hex(tx);
            self.sendrawtransaction(tx_hex);
        }
    }
}

//
// Exercise 3: BroadcasterInterface (END)
//

//----------------------------------------------------------------------

//
// Exercise 4: FeeEstimator (START)
//

pub struct FeeRateEstimate {
    pub target_1_block: u32,
    pub target_3_block: u32,
    pub target_6_block: u32,
    pub target_144_block: u32,
    pub target_1008_block: u32,
}


impl BitcoinClient {
    fn rpc_estimate_smart_fee(&self) -> FeeRateEstimate {
        FeeRateEstimate {
            target_1_block: 6,
            target_3_block: 5,
            target_6_block: 5,
            target_144_block: 4,
            target_1008_block: 2,
        }
    }

}

impl FeeEstimator for BitcoinClient {
    fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
        let fee_rates = self.rpc_estimate_smart_fee();
        match confirmation_target {
            ConfirmationTarget::MaximumFeeEstimate => fee_rates.target_1_block * 250 as u32,

            ConfirmationTarget::UrgentOnChainSweep => fee_rates.target_1_block * 250 as u32,

            ConfirmationTarget::OutputSpendingFee => fee_rates.target_1_block * 250 as u32,

            ConfirmationTarget::NonAnchorChannelFee => fee_rates.target_6_block * 250 as u32,

            ConfirmationTarget::AnchorChannelFee => fee_rates.target_1008_block * 250 as u32,

            ConfirmationTarget::ChannelCloseMinimum => fee_rates.target_1008_block * 250 as u32,

            ConfirmationTarget::MinAllowedNonAnchorChannelRemoteFee => fee_rates.target_1008_block * 250 as u32,

            ConfirmationTarget::MinAllowedAnchorChannelRemoteFee => fee_rates.target_1008_block * 250 as u32,
        }
    }
}


//
// Exercise 4: FeeEstimator (END)
//