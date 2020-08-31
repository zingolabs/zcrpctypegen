use base64::encode;
use std::fmt::Debug;
use std::future::Future;
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
    let host = get_var("ZCASHRPC_TEST_HOST");
    let auth = encode(get_var("ZCASHRPC_TEST_AUTH"));
    Client::new(host, auth)
}

fn get_var(name: &str) -> String {
    std::env::var(name).expect(&format!(
        "Environment variable {} must be set to enable smoke tests.",
        name
    ))
}
