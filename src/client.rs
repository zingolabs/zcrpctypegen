//! Includes both `Client` and all of the RPC response types.
#[macro_use]
mod defapi;
pub mod subcomponents;

use self::subcomponents::{Consensus, NetworkUpgradeDesc, Softfork, ValuePool};
use crate::{ResponseResult, ZecAmount};
use reqwest;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::ops::RangeFrom;

/// A `Client` is used to make multiple requests to a specific zcashd RPC server. Requests are invoked by async methods that correspond to `zcashd` RPC API method names with request-specific parameters. Each such method has an associated response type.
pub struct Client {
    url: String,
    auth: String,
    reqcli: reqwest::Client,
    idit: RangeFrom<u64>,
}

impl Client {
    /// Construct a new `Client` with connection & authentication info.
    /// - `hostport` is a host/ip with an optional `:PORT` appended.
    /// - `authcookie` is the contents of `~/.zcash/.cookie`.
    pub fn new(hostport: String, authcookie: String) -> Client {
        Client {
            url: format!("http://{}/", hostport),
            auth: format!("Basic {}", authcookie),
            reqcli: reqwest::Client::new(),
            idit: (0..),
        }
    }
}

def_api_method! {
    getinfo() -> GetInfoResponse {
balance: ZecAmount,
             blocks: u64,
             connections: u64,
             difficulty: f64,
             errors: String,
             keypoololdest: u64,
             keypoolsize: u64,
             paytxfee: ZecAmount,
             protocolversion: u64,
             proxy: String,
             relayfee: ZecAmount,
             testnet: bool,
             timeoffset: u64,
             version: u64,
             walletversion: u64
    }
}

def_api_method! {
    getblockchaininfo() -> GetBlockChainInfoResponse {
chain: String,
           blocks: u64,
           headers: u64,
           bestblockhash: String,
           difficulty: f64,
           verificationprogress: f64,
           chainwork: String,
           pruned: bool,
           size_on_disk: u64,
           commitments: u64,
           valuePools: Vec<ValuePool>,
           softforks: Vec<Softfork>,
           upgrades: std::collections::HashMap<String, NetworkUpgradeDesc>,
           consensus: Consensus,
           pruneheight: Option<u64>,
           fullyNotified: Option<bool>
    }
}

impl Client {
    fn make_request<R>(
        &mut self,
        method: &'static str,
        args: Vec<serde_json::Value>,
    ) -> impl Future<Output = ResponseResult<R>>
    where
        R: DeserializeOwned,
    {
        use crate::{
            envelope::{RequestEnvelope, ResponseEnvelope},
            json,
        };

        let id = self.idit.next().unwrap();
        let sendfut = self
            .reqcli
            .post(&self.url)
            .header("Authorization", &self.auth)
            .body(&RequestEnvelope::wrap(id, method, args))
            .send();
        async move {
            let reqresp = sendfut.await?;
            let text = reqresp.text().await?;
            let respenv: ResponseEnvelope = json::parse_value(json::parse_string(text)?)?;
            let resp = respenv.unwrap(id)?;
            Ok(resp)
        }
    }
}
