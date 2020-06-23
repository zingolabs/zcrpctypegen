use serde_json;

#[derive(Debug)]
pub struct Error {
    detail: serde_json::Error,
    input: MalformedInput,
}

#[derive(Debug)]
pub enum MalformedInput {
    Json(String),
    Structure(serde_json::Value),
}

pub fn parse_string<T>(s: String) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    use serde_json::{from_str, from_value, Value};

    let val: Value = from_str(&s).map_err(move |e| Error {
        detail: e,
        input: MalformedInput::Json(s),
    })?;

    let obj = from_value(val.clone()).map_err(move |e| Error {
        detail: e,
        input: MalformedInput::Structure(val),
    })?;

    Ok(obj)
}
