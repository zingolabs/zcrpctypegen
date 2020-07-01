mod envelope;

use self::envelope::RequestEnvelope;
use crate::msg::Request;
use reqwest;
use std::ops::RangeFrom;

pub struct Client {
    url: String,
    auth: String,
    reqcli: reqwest::Client,
    idit: RangeFrom<u64>,
}

#[derive(derive_more::From, Debug)]
pub enum Error<R> {
    Response(self::envelope::ResponseError<R>),
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

    pub async fn request<'a, R>(
        &mut self,
        request: &'a R,
    ) -> Result<R::Response, Error<R::Response>>
    where
        R: Request,
    {
        use self::envelope::ResponseEnvelope;
        use crate::json;

        let id = self.idit.next().unwrap();
        let reqresp = self
            .reqcli
            .post(&self.url)
            .header("Authorization", &self.auth)
            .body(&RequestEnvelope::wrap(id, request))
            .send()
            .await?;

        let text = reqresp.text().await?;
        let respenv: ResponseEnvelope<R::Response> = json::parse_string(text)?;
        let resp = respenv.unwrap(id)?;
        Ok(resp)
    }
}
