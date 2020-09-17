#[path = "src/client/utils.rs"]
mod utils;

use serde::{Deserialize, Serialize};
use std::io::Write as _;

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestEnvelope {
    id: u64,
    method: &'static str,
    params: Vec<serde_json::Value>,
}

impl<'a> From<&'a RequestEnvelope> for reqwest::Body {
    fn from(re: &'a RequestEnvelope) -> reqwest::Body {
        use serde_json::to_string_pretty;

        reqwest::Body::from(to_string_pretty(re).unwrap())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut id = 0..;
    let client = reqwest::Client::new();
    let rpc_calls = vec!["GetInfoResponse", "GetBlockChainInfoResponse"];
    let _json_dir = std::fs::DirBuilder::new().create("json_data");
    for call in rpc_calls {
        let response = client
            .post(&format!("http://{}/", crate::utils::get_zcashd_port()))
            .header(
                "Authorization",
                format!(
                    "Basic {}",
                    base64::encode(&crate::utils::get_cookie(true)?)
                ),
            )
            .body(&RequestEnvelope {
                id: id.next().unwrap(),
                method: &get_call_name(call),
                params: Vec::new(),
            })
            .send();
        let text = response.await?.text().await?;
        let json: serde_json::Value = serde_json::de::from_str(&text).unwrap();
        let result = json.as_object().unwrap().get("result").unwrap();
        let mut temp_file =
            std::fs::File::create(format!("json_data/{}.json", call))?;
        temp_file.write(serde_json::to_string_pretty(result)?.as_bytes())?;
    }
    Ok(())
}

fn get_call_name(call_response: &str) -> &'static str {
    Box::leak(
        call_response
            .strip_suffix("Response")
            .unwrap()
            .to_lowercase()
            .into_boxed_str(),
    )
}
