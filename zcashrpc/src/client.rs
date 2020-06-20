use crate::msg::Request;
use reqwest;

pub struct Client {
    url: String,
    auth: String,
    reqcli: reqwest::Client,
}

#[derive(derive_more::From, Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Json(crate::json::Error),
}

impl Client {
    pub fn new(host: String, authcookie: String) -> Client {
        Client {
            url: format!("http://{}/", host),
            auth: format!("Basic {}", authcookie),
            reqcli: reqwest::Client::new(),
        }
    }

    pub async fn request<'a, R>(&self, request: &'a R) -> Result<R::Response, Error>
    where
        R: Request,
        reqwest::Body: From<&'a R>,
    {
        use crate::json;

        let reqresp = self
            .reqcli
            .post(&self.url)
            .header("Authorization", &self.auth)
            .body(request)
            .send()
            .await?;

        let text = reqresp.text().await?;
        let resp = json::parse_string(text)?;
        Ok(resp)
    }
}
