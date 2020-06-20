use crate::msg::Request;
use reqwest;

pub struct Client {
    url: String,
    auth: String,
    reqcli: reqwest::Client,
}

impl Client {
    pub fn new(host: String, authcookie: String) -> Client {
        Client {
            url: format!("http://{}/", host),
            auth: format!("Basic {}", authcookie),
            reqcli: reqwest::Client::new(),
        }
    }

    pub async fn request<'a, R>(&self, request: &'a R) -> Result<R::Response, reqwest::Error>
    where
        R: Request,
        reqwest::Body: From<&'a R>,
    {
        let reqresp = self
            .reqcli
            .post(&self.url)
            .header("Authorization", &self.auth)
            .body(request)
            .send()
            .await?;

        let resp = reqresp.json().await?;

        Ok(resp)
    }
}
