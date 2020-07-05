use crate::ResponseResult;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub fn parse_string(s: String) -> ResponseResult<Value> {
    use crate::error::JsonRpcViolation::MalformedJson;
    use serde_json::from_str;

    let val: Value = from_str(&s).map_err(move |e| MalformedJson {
        input_text: s,
        reason: e,
    })?;

    Ok(val)
}

pub fn parse_value<R>(val: Value) -> ResponseResult<R>
where
    R: DeserializeOwned,
{
    use crate::error::UnexpectedResponse;
    use serde_json::from_value;

    let obj = from_value(val.clone()).map_err(move |e| UnexpectedResponse {
        structure: val,
        reason: e,
    })?;

    Ok(obj)
}
