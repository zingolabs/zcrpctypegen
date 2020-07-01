use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestEnvelope {
    id: u64,
    method: &'static str,
    params: Vec<serde_json::Value>,
}

#[derive(Debug)]
pub enum ResponseError<R> {
    UnexpectedServerId { client: u64, server: u64 },
    NoResultOrError,
    ResultAndError { result: R, error: ServerError },
    Server(ServerError),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseEnvelope<R> {
    id: u64,
    result: Option<R>,
    error: Option<ServerError>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerError {
    code: i64,
    message: String,
}

impl<'a> From<&'a RequestEnvelope> for reqwest::Body {
    fn from(re: &'a RequestEnvelope) -> reqwest::Body {
        use serde_json::to_string_pretty;

        reqwest::Body::from(to_string_pretty(re).unwrap())
    }
}

impl RequestEnvelope {
    pub fn wrap(id: u64, method: &'static str, params: Vec<serde_json::Value>) -> RequestEnvelope {
        RequestEnvelope {
            id: id,
            method: method,
            params: params,
        }
    }
}

impl<R> ResponseEnvelope<R>
where
    R: DeserializeOwned,
{
    pub fn unwrap(self, clientid: u64) -> Result<R, ResponseError<R>> {
        if self.id != clientid {
            Err(ResponseError::UnexpectedServerId {
                client: clientid,
                server: self.id,
            })
        } else {
            match (self.result, self.error) {
                (None, None) => Err(ResponseError::NoResultOrError),
                (Some(r), Some(e)) => Err(ResponseError::ResultAndError {
                    result: r,
                    error: e,
                }),
                (Some(r), None) => Ok(r),
                (None, Some(e)) => Err(ResponseError::Server(e)),
            }
        }
    }
}
