use tokio;
use zcashrpc;
use zcashrpc::msg::getinfo;

#[derive(derive_more::From, Debug)]
pub enum Error {
    ZcashRPC(zcashrpc::Error<getinfo::Response>),
    Var(std::env::VarError),
}

#[tokio::test]
async fn getinfo() -> Result<(), Error> {
    let mut client = make_client()?;
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
