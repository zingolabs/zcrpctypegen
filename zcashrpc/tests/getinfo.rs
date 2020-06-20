use tokio;
use zcashrpc;

#[derive(derive_more::From, Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Var(std::env::VarError),
}

#[tokio::test]
async fn getinfo() {
    getinfo_inner().await.unwrap()
}

async fn getinfo_inner() -> Result<(), Error> {
    use zcashrpc::msg::getinfo;

    let client = make_client()?;
    let resp = client.request(&getinfo::Request {}).await?;
    assert_eq!("SSS", &format!("{:?}", &resp));
    Ok(())
}

fn make_client() -> Result<zcashrpc::Client, Error> {
    use std::env::var;

    let host = var("ZCASHRPC_TEST_HOST")?;
    let auth = var("ZCASHRPC_TEST_AUTH")?;
    Ok(zcashrpc::Client::new(host, auth))
}
