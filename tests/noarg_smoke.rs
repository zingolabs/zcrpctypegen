use tokio;
use zcashrpc;

#[derive(derive_more::From, Debug)]
pub enum Error {
    ZcashRPC(zcashrpc::Error),
    Var(std::env::VarError),
}

pub fn make_client() -> Result<zcashrpc::Client, std::env::VarError> {
    use std::env::var;

    let host = var("ZCASHRPC_TEST_HOST")?;
    let auth = var("ZCASHRPC_TEST_AUTH")?;
    Ok(zcashrpc::Client::new(host, auth))
}

macro_rules! def_tests {
    ( $( $name:ident ),* ) => {
        $(
        #[tokio::test]
        async fn $name() -> Result<(), Error> {
            let mut client = make_client()?;
            let resp = client.$name().await?;
            println!("{:?}", &resp);
            Ok(())
        }
        )*
    }
}

def_tests!(getinfo, getblockchaininfo);
