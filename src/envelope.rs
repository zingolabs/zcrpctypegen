pub use crate::client::utils::RequestEnvelope;
use crate::{error::ResponseError, ResponseResult};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseEnvelope {
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<ResponseError>,
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

    fn unwrap_internal(
        self,
        clientid: u64,
    ) -> ResponseResult<serde_json::Value> {
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
