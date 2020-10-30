//! Sub-components of response messages.

pub mod getblockchaininfo {
    use crate::ZecAmount;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct GetBlockChainInfoResponse {
        pub chain: String,
        pub blocks: u64,
        pub headers: u64,
        pub bestblockhash: String,
        pub difficulty: f64,
        pub verificationprogress: f64,
        pub chainwork: String,
        pub pruned: bool,
        pub size_on_disk: u64,
        pub commitments: u64,
        #[serde(rename = "valuePools")]
        pub value_pools: Vec<ValuePool>,
        pub softforks: Vec<Softfork>,
        pub upgrades: std::collections::HashMap<String, NetworkUpgradeDesc>,
        pub consensus: Consensus,
        pub pruneheight: Option<u64>,
        #[serde(rename = "fullyNotified")]
        pub fully_notified: Option<bool>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ValuePool {
        pub id: String,
        pub monitored: bool,
        #[serde(rename = "chainValue")]
        pub chain_value: Option<ZecAmount>,
        #[serde(rename = "chainValueZat")]
        pub chain_value_zat: Option<u64>,
        #[serde(rename = "valueDelta")]
        pub value_delta: Option<ZecAmount>,
        #[serde(rename = "valueDeltaZat")]
        pub value_delta_zat: Option<i64>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Softfork {
        pub id: String,
        pub version: i64,
        pub enforce: SoftforkMajorityDesc,
        pub reject: SoftforkMajorityDesc,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct SoftforkMajorityDesc {
        pub status: bool,
        pub found: i64,
        pub required: i64,
        pub window: serde_json::Value, // FIXME
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct NetworkUpgradeDesc {
        pub name: String,
        pub activationheight: u64,
        pub status: String, // FIXME: enum-ify
        pub info: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Consensus {
        pub chaintip: String,
        pub nextblock: String,
    }
}

pub use self::getblockchaininfo::GetBlockChainInfoResponse;
