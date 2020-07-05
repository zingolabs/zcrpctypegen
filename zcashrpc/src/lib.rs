//! An asynchronous zcashd RPC client.
mod envelope;
mod json;
#[macro_use]
mod defapi;
pub mod error;

use reqwest;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::ops::RangeFrom;

/// A `Client` is used to make multiple requests to a specific zcashd RPC server. Requests are invoked by async methods that correspond to `zcashd` RPC API method names with request-specific parameters. Each such method has an associated response type.
pub struct Client {
    url: String,
    auth: String,
    reqcli: reqwest::Client,
    idit: RangeFrom<u64>,
}

#[doc(inline)]
pub use error::{Error, ResponseResult};

/// The `ZecAmount` type alias is used to document where ZEC-denominated fields are used. Note that this does not represent Zatoshi-denominated units.
pub type ZecAmount = f64;

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

    async fn make_request<R>(
        &mut self,
        method: &'static str,
        args: Vec<serde_json::Value>,
    ) -> ResponseResult<R>
    where
        R: DeserializeOwned,
    {
        use crate::envelope::{RequestEnvelope, ResponseEnvelope};

        let id = self.idit.next().unwrap();
        let reqresp = self
            .reqcli
            .post(&self.url)
            .header("Authorization", &self.auth)
            .body(&RequestEnvelope::wrap(id, method, args))
            .send()
            .await?;
        let text = reqresp.text().await?;
        let respenv: ResponseEnvelope = json::parse_value(json::parse_string(text)?)?;
        let resp = respenv.unwrap(id)?;
        Ok(resp)
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
