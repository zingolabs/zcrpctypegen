use dirs::home_dir;
use std::fmt::Debug;
use std::fs::File;
use std::future::Future;
use std::io::{self, Read};
use tokio;
use zcashrpc::Client;

#[derive(Debug)]
struct TestsFailed;

#[tokio::test]
async fn main() -> Result<(), TestsFailed> {
    let mut runner = Runner::new();

    runner.run("getinfo", || make_client().getinfo()).await;
    runner
        .run("getblockchaininfo", || make_client().getblockchaininfo())
        .await;

    runner.finish()
}

struct Runner {
    tests: u64,
    failures: u64,
}

impl Runner {
    fn new() -> Runner {
        Runner {
            tests: 0,
            failures: 0,
        }
    }

    async fn run<F, Fut, R, E>(&mut self, name: &str, test: F)
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<R, E>>,
        R: Debug,
        E: Debug,
    {
        self.tests += 1;
        println!("=== smoke test {}... ", name);
        match test().await {
            Ok(r) => println!("ok.\n{:#?}\n", r),
            Err(e) => {
                self.failures += 1;
                println!("FAIL:\n  {:#?}\n", e);
            }
        }
    }

    fn finish(self) -> Result<(), TestsFailed> {
        println!("=== {} tests with {} failures.", self.tests, self.failures);
        if self.failures == 0 {
            Ok(())
        } else {
            Err(TestsFailed)
        }
    }
}

fn make_client() -> Client {
    let host = std::env::var("ZCASHRPC_TEST_HOST")
        .unwrap_or(String::from("127.0.0.1:18232".to_string()));
    let auth = std::env::var("ZCASHRPC_TEST_AUTH")
        .or_else(get_cookie)
        .expect("cookie lookup failed");
    Client::new(host, auth)
}

fn get_cookie(error: std::env::VarError) -> std::io::Result<String> {
    let log_msg = format!(
        "Invalid or no value passed to environment {} {}. {}",
        "variable ZCASHRPC_TEST_AUTH. Details:",
        error,
        "Defaulting to cookie lookup."
    );
    dbg!(log_msg);
    let mut cookie_path = match home_dir() {
        Some(x) => x,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
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

    let mut cookie_file = File::open(cookie_path)?;
    let mut cookie_string = String::new();
    cookie_file.read_to_string(&mut cookie_string)?;
    Ok(cookie_string)
}
