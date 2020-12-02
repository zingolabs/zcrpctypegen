//! Sub-components of response messages.

use crate::ZecAmount;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GetInfoResponse {
    balance: ZecAmount,
    blocks: Decimal,
    connections: Decimal,
    difficulty: Decimal,
    errors: String,
    keypoololdest: Decimal,
    keypoolsize: Decimal,
    paytxfee: ZecAmount,
    protocolversion: Decimal,
    proxy: String,
    relayfee: ZecAmount,
    testnet: bool,
    timeoffset: Decimal,
    version: Decimal,
    walletversion: Decimal,
}

pub mod getblockchaininfo {
    use crate::ZecAmount;
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct GetBlockChainInfoResponse {
        chain: String,
        blocks: Decimal,
        headers: Decimal,
        bestblockhash: String,
        difficulty: Decimal,
        verificationprogress: Decimal,
        chainwork: String,
        pruned: bool,
        size_on_disk: Decimal,
        commitments: Decimal,
        #[serde(rename = "valuePools")]
        value_pools: Vec<ValuePool>,
        softforks: Vec<Softfork>,
        upgrades: std::collections::HashMap<String, NetworkUpgradeDesc>,
        consensus: Consensus,
        pruneheight: Option<Decimal>,
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
        pub chain_value_zat: Option<Decimal>,
        #[serde(rename = "valueDelta")]
        pub value_delta: Option<ZecAmount>,
        #[serde(rename = "valueDeltaZat")]
        pub value_delta_zat: Option<Decimal>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Softfork {
        pub id: String,
        pub version: Decimal,
        pub enforce: SoftforkMajorityDesc,
        pub reject: SoftforkMajorityDesc,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct SoftforkMajorityDesc {
        pub status: bool,
        pub found: Decimal,
        pub required: Decimal,
        pub window: serde_json::Value, // FIXME
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct NetworkUpgradeDesc {
        pub name: String,
        pub activationheight: Decimal,
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
pub type ZGetNewAddressResponse = String;
pub type GenerateResponse = Vec<String>;
