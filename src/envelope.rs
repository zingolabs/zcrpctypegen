use crate::{error::ResponseError, ResponseResult};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestEnvelope {
    id: u64,
    method: &'static str,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseEnvelope {
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<ResponseError>,
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

impl ResponseEnvelope {
    pub fn unwrap<R>(self, clientid: u64) -> ResponseResult<R>
    where
        R: DeserializeOwned,
    {
        use crate::json;

        let jv = self.unwrap_internal(clientid)?;
        json::parse_value(jv)
    }

    fn unwrap_internal(self, clientid: u64) -> ResponseResult<serde_json::Value> {
        use crate::{
            error::JsonRpcViolation::*,
            Error::{JsonRpcViolation, Response},
        };

        if self.id != clientid {
            Err(JsonRpcViolation(UnexpectedServerId {
                client: clientid,
                server: self.id,
            }))
        } else {
            match (self.result, self.error) {
                (None, None) => Err(JsonRpcViolation(NoResultOrError)),
                (Some(r), Some(e)) => Err(JsonRpcViolation(ResultAndError {
                    result: r,
                    error: e,
                })),
                (Some(r), None) => Ok(r),
                (None, Some(e)) => Err(Response(e)),
            }
        }
    }
}
