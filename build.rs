#[path = "src/client/utils.rs"]
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //temporary POC, this will likely be replaced by a config-file in the
    //short-term, and hopefully eventually a query against zcashd itself
    let rpc_response_names: Vec<(&'static str, Vec<serde_json::Value>)> = vec![
        ("GetInfoResponse", Vec::new()),
        ("GetBlockChainInfoResponse", Vec::new()),
        ("GenerateResponse", vec![2.into()]),
        ("ZGetNewAddressResponse", Vec::new()),
    ];

    //This assignment prevents the compiler from complaining if the file
    //already exists. If the file already exists, this is correctly a noop,
    //and not an error we need to be informed of.
    let _json_dir = std::fs::DirBuilder::new().create("json_data");
    for response_name in rpc_response_names {
        let (_, response) = utils::InnerCli::new(
            utils::get_zcashd_port(),
            utils::get_cookie(true)?,
        )
        .procedure_call(get_call_name(response_name.0), response_name.1);
        let text = response.await?.text().await?;
        let json: serde_json::Value = serde_json::de::from_str(&text).unwrap();
        let result = json.as_object().unwrap().get("result").unwrap();

        use std::io::Write as _;
        let mut data_file = std::fs::File::create(format!(
            "json_data/{}.json",
            response_name.0
        ))?;
        data_file.write(serde_json::to_string_pretty(result)?.as_bytes())?;
    }
    Ok(())
}

fn get_call_name(response_name: &str) -> &'static str {
    //We need to leak in order to get a &'static str at runtime, which is
    //needed by our RequestEnvelope. It's possible we can rewrite it to not
    //need a 'static, but that's a problem for future us.
    let mut call_name = response_name
        .strip_suffix("Response")
        .unwrap()
        .to_lowercase();
    if call_name.starts_with('z') {
        call_name.insert(1, '_');
    }
    Box::leak(call_name.into_boxed_str())
}
