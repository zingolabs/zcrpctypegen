mod envelope;
mod json;
#[macro_use]
mod defapi;

use reqwest;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::ops::RangeFrom;

pub use envelope::ResponseEnvelopeError;
pub type ZecAmount = f64;

pub struct Client {
    url: String,
    auth: String,
    reqcli: reqwest::Client,
    idit: RangeFrom<u64>,
}

#[derive(derive_more::From, Debug)]
pub enum Error<R> {
    Response(ResponseEnvelopeError<R>),
    Reqwest(reqwest::Error),
    Json(crate::json::Error),
}

impl Client {
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
    ) -> Result<R, Error<R>>
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
        let respenv: ResponseEnvelope<R> = json::parse_string(text)?;
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
