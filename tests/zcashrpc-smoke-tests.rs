fn make_client() -> zcashrpc::Client {
    let host = std::env::var("ZCASHRPC_TEST_HOST")
        .unwrap_or(String::from("127.0.0.1:18232".to_string()));
    let auth = std::env::var("ZCASHRPC_TEST_AUTH").unwrap_or_else(|_| {
        let cookie_path = match std::env::var("REGTEST") {
            Ok(_) => "~/.zcash/regtest/.cookie",
            Err(_) => "~/.zcash/.cookie",
        };
        let mut cookie_file = std::fs::File::open(cookie_path)
            .expect(&format!("no cookie found in {}", cookie_path));
        let mut cookie_string = String::new();
        use std::io::Read;
        cookie_file
            .read_to_string(&mut cookie_string)
            .expect("Failed to read cookie");
        cookie_string
    });
    zcashrpc::Client::new(host, auth)
}
macro_rules! run_smoketest {
    ($x:ident) => {
        #[tokio::test]
        async fn $x() {
            let _res = make_client().$x().await.unwrap();
        }
    }
}

run_smoketest!(getinfo);
run_smoketest!(getblockchaininfo);
run_smoketest!(z_getnewaddress);
