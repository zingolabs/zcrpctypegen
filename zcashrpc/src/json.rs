use serde_json;

#[derive(Debug)]
pub struct Error {
    detail: serde_json::Error,
    input: String,
}

pub fn parse_string<T>(s: String) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(&s).map_err(move |e| Error {
        detail: e,
        input: s,
    })
}
