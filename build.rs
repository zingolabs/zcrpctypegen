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
            method: "getinfo",
            params: Vec::new(),
        })
        .send();
    let text = response.await?.text().await?;
    let json: serde_json::Value = serde_json::de::from_str(&text).unwrap();
    let result = json.as_object().unwrap().get("result").unwrap();
    let mut temp_file = std::fs::File::create("temp_json_data.json")?;
    temp_file.write(serde_json::to_string_pretty(result)?.as_bytes())?;
    println!("{}", text);

    Ok(())
}
