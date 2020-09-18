#[path = "src/client/utils.rs"]
mod utils;

//Copied from envelope.rs to not get more types and deps than we need
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RequestEnvelope {
    id: u64,
    method: &'static str,
    params: Vec<serde_json::Value>,
}

//Copied from envelope.rs to not get more types and deps than we need
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

    //temporary POC, this will likely be replaced by a config-file in the
    //short-term, and hopefully eventually a query against zcashd itself
    let rpc_response_names =
        vec!["GetInfoResponse", "GetBlockChainInfoResponse"];

    //This assignment prevents the compiler from complaining if the file
    //already exists. If the file already exists, this is correctly a noop,
    //and not an error we need to be informed of.
    let _json_dir = std::fs::DirBuilder::new().create("json_data");
    for response_name in rpc_response_names {
        //Logic mostly copied from client.rs, as we don't want to inherit deps
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
                method: &get_call_name(response_name),
                params: Vec::new(),
            })
            .send();
        let text = response.await?.text().await?;
        let json: serde_json::Value = serde_json::de::from_str(&text).unwrap();
        let result = json.as_object().unwrap().get("result").unwrap();

        use std::io::Write as _;
        let mut data_file =
            std::fs::File::create(format!("json_data/{}.json", response_name))?;
        data_file.write(serde_json::to_string_pretty(result)?.as_bytes())?;
    }
    Ok(())
}

fn get_call_name(response_name: &str) -> &'static str {
    //We need to leak in order to get a &'static str at runtime, which is
    //needed by our RequestEnvelope. It's possible we can rewrite it to not
    //need a 'static, but that's a problem for future us.
    Box::leak(
        response_name
            .strip_suffix("Response")
            .unwrap()
            .to_lowercase()
            .into_boxed_str(),
    )
}
