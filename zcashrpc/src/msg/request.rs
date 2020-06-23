use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait Request: Serialize {
    type Response: DeserializeOwned;

    fn name() -> &'static str;

    fn params(&self) -> Vec<serde_json::Value>;
}

impl<'a, T: Request> Request for &'a T {
    type Response = T::Response;

    fn name() -> &'static str {
        T::name()
    }

    fn params(&self) -> Vec<serde_json::Value> {
        (*self).params()
    }
}
