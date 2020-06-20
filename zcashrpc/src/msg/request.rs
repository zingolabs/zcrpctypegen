use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait Request: Serialize {
    type Response: DeserializeOwned;

    fn name() -> &'static str;
}
