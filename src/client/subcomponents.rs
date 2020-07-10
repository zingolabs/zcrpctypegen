//! Sub-components of response messages.

use crate::ZecAmount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ValuePool {
    pub id: String,
    pub monitored: bool,
    pub chainValue: Option<ZecAmount>,
    pub chainValueZat: Option<u64>,
    pub valueDelta: Option<ZecAmount>,
    pub valueDeltaZat: Option<i64>,
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
