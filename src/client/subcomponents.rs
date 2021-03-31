//! Sub-components of response messages.

pub mod getinfo {
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct GetinfoResponse {
        balance: crate::ZecAmount,
        blocks: rust_decimal::Decimal,
        connections: rust_decimal::Decimal,
        difficulty: rust_decimal::Decimal,
        errors: String,
        keypoololdest: rust_decimal::Decimal,
        keypoolsize: rust_decimal::Decimal,
        paytxfee: crate::ZecAmount,
        protocolversion: rust_decimal::Decimal,
        proxy: String,
        relayfee: crate::ZecAmount,
        testnet: bool,
        timeoffset: rust_decimal::Decimal,
        version: rust_decimal::Decimal,
        walletversion: rust_decimal::Decimal,
    }
}

pub mod getblockchaininfo {

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct GetblockchaininfoResponse {
        chain: String,
        blocks: rust_decimal::Decimal,
        headers: rust_decimal::Decimal,
        bestblockhash: String,
        difficulty: rust_decimal::Decimal,
        verificationprogress: rust_decimal::Decimal,
        chainwork: String,
        pruned: bool,
        size_on_disk: rust_decimal::Decimal,
        commitments: rust_decimal::Decimal,
        #[serde(rename = "valuePools")]
        value_pools: Vec<ValuePool>,
        softforks: Vec<Softfork>,
        upgrades: std::collections::HashMap<String, NetworkUpgradeDesc>,
        consensus: Consensus,
        pruneheight: Option<rust_decimal::Decimal>,
        #[serde(rename = "fullyNotified")]
        pub fully_notified: Option<bool>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct ValuePool {
        pub id: String,
        pub monitored: bool,
        #[serde(rename = "chainValue")]
        pub chain_value: Option<crate::ZecAmount>,
        #[serde(rename = "chainValueZat")]
        pub chain_value_zat: Option<rust_decimal::Decimal>,
        #[serde(rename = "valueDelta")]
        pub value_delta: Option<crate::ZecAmount>,
        #[serde(rename = "valueDeltaZat")]
        pub value_delta_zat: Option<rust_decimal::Decimal>,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Softfork {
        pub id: String,
        pub version: rust_decimal::Decimal,
        pub enforce: SoftforkMajorityDesc,
        pub reject: SoftforkMajorityDesc,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct SoftforkMajorityDesc {
        pub status: bool,
        pub found: rust_decimal::Decimal,
        pub required: rust_decimal::Decimal,
        pub window: serde_json::Value, // FIXME
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct NetworkUpgradeDesc {
        pub name: String,
        pub activationheight: rust_decimal::Decimal,
        pub status: String, // FIXME: enum-ify
        pub info: String,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Consensus {
        pub chaintip: String,
        pub nextblock: String,
    }
}
pub mod z_getnewaddress {
    pub type ZGetnewaddressResponse = String;
}
pub mod generate {
    pub type GenerateResponse = Vec<String>;
}
