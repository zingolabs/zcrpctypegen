fn make_client() -> zcashrpc::Client {
    let host = std::env::var("ZCASHRPC_TEST_HOST")
        .unwrap_or(String::from("127.0.0.1:18232".to_string()));
    let auth = std::env::var("ZCASHRPC_TEST_AUTH")
        .or_else(get_cookie)
        .expect("cookie lookup failed");
    zcashrpc::Client::new(host, auth)
}
macro_rules! run_smoketest {
    ($x:ident) => {
        #[tokio::test]
        async fn $x() {
            let _res = make_client().$x().await.unwrap();
        }
    };
}

run_smoketest!(getinfo);
run_smoketest!(getblockchaininfo);
run_smoketest!(z_getnewaddress);

fn get_cookie(error: std::env::VarError) -> std::io::Result<String> {
    let log_msg = format!(
        "Invalid or no value passed to environment {} {}. {}",
        "variable ZCASHRPC_TEST_AUTH. Details:",
        error,
        "Defaulting to cookie lookup."
    );
    dbg!(log_msg);
    let mut cookie_path = match dirs::home_dir() {
        Some(x) => x,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "{} {} {}",
                    "Could not find your home directory.",
                    "Please pass the contents of ~/.zcash/.cookie to the",
                    "enviroment variable ZCASH_TEST_AUTH."
                ),
            ))
        }
    };

    cookie_path.push(".zcash");
    if std::env::var("REGTEST").is_ok() {
        cookie_path.push("regtest");
    }
    cookie_path.push(".cookie");

    let mut cookie_file = std::fs::File::open(cookie_path)?;
    let mut cookie_string = String::new();
    use std::io::Read as _;
    cookie_file.read_to_string(&mut cookie_string)?;
    Ok(cookie_string)
}
